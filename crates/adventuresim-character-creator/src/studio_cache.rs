//! What the studio keeps between regenerations, so that an edit rebuilds only
//! what it changes: the body while its shape stands, each worn piece's fit
//! while its design and the body stand, its finish while its construction and
//! decoration stand too, and the drape while nothing it lies on changed.

use std::collections::HashMap;
use std::sync::Arc;

use super::*;
use adventuresim_character_creator::{
    garment::PlateLining,
    inventory::{FittedPiece, Loadout},
};
use parametric_equipment::{Finish, SelectedArmor};

/// A cache key: the serialized inputs of what it names.
type Key = Vec<u8>;

#[derive(Resource, Default)]
pub(super) struct StudioCache {
    body: Option<(Key, Arc<GeneratedCharacter>)>,
    fits: HashMap<Key, GeneratedArmor>,
    finishes: HashMap<Key, Finish>,
    /// The lining of the worn plate, by the fits it was made of.
    lining: Option<(Vec<Key>, Option<Arc<PlateLining>>)>,
    /// The inputs of the last drape requested.
    drape: Option<Key>,
}

impl StudioCache {
    /// The body `recipe` describes, generated again only when its shape or
    /// the model changed. A new body drops every fit made on the old one.
    pub(super) fn body(
        &mut self,
        model: &BodyModel,
        recipe: &CharacterRecipe,
    ) -> Result<Arc<GeneratedCharacter>> {
        let key = serde_json::to_vec(&(
            model.lod,
            &recipe.proportions,
            &recipe.identity,
            &recipe.expression,
        ))?;
        if let Some((cached, body)) = &self.body
            && *cached == key
        {
            return Ok(body.clone());
        }
        let body = Arc::new(generate_character(model, recipe).context("Generation failed")?);
        self.fits.clear();
        self.finishes.clear();
        self.lining = None;
        self.drape = None;
        self.body = Some((key, body.clone()));
        Ok(body)
    }

    /// Every worn piece fitted and finished, reusing what its inputs allow.
    /// Pieces no longer worn are forgotten.
    pub(super) fn armor<'a>(
        &mut self,
        model: &BodyModel,
        generated: &GeneratedCharacter,
        loadout: &Loadout<'a>,
    ) -> Result<Vec<SelectedArmor<'a>>> {
        let mut fits = HashMap::new();
        let mut finishes = HashMap::new();
        let mut armor = Vec::with_capacity(loadout.fitted.len());
        for piece in &loadout.fitted {
            let fit_key = fit_key(piece)?;
            let fitted = match self.fits.remove(&fit_key) {
                Some(fitted) => fitted,
                None => parametric_equipment::fit(model, generated, piece, &[])?,
            };
            let finish_key = finish_key(&fit_key, piece)?;
            let finish = match self.finishes.remove(&finish_key) {
                Some(finish) => finish,
                None => parametric_equipment::finish(piece, fitted.clone())?,
            };
            armor.push(SelectedArmor::new(piece, finish.clone()));
            fits.insert(fit_key, fitted);
            finishes.insert(finish_key, finish);
        }
        self.fits = fits;
        self.finishes = finishes;
        Ok(armor)
    }

    /// The lining of the worn rigid plate, as fitted before it is built of
    /// small plates, so that only a change of fit moves the cloth under it.
    pub(super) fn lining(&mut self, loadout: &Loadout<'_>) -> Result<Option<Arc<PlateLining>>> {
        let rigid = loadout
            .fitted
            .iter()
            .filter(|piece| parametric_equipment::is_rigid(piece))
            .collect::<Vec<_>>();
        let keys = rigid
            .iter()
            .map(|piece| fit_key(piece))
            .collect::<Result<Vec<_>>>()?;
        if let Some((cached, lining)) = &self.lining
            && *cached == keys
        {
            return Ok(lining.clone());
        }
        let lining =
            PlateLining::new(keys.iter().filter_map(|key| self.fits.get(key))).map(Arc::new);
        self.lining = Some((keys, lining.clone()));
        Ok(lining)
    }

    /// Whether the drape of `loadout` on the cached body under `lining`
    /// differs from the last one requested; records it when it does.
    pub(super) fn drape_changed(&mut self, loadout: &Loadout<'_>) -> Result<bool> {
        let draped = loadout
            .draped
            .iter()
            .map(|piece| serde_json::to_vec(&(piece.id, piece.selection, piece.drape)))
            .collect::<serde_json::Result<Vec<_>>>()?;
        let key = serde_json::to_vec(&(
            self.body.as_ref().map(|(key, _)| key),
            self.lining.as_ref().map(|(keys, _)| keys),
            draped,
        ))?;
        let changed = self.drape.as_ref() != Some(&key);
        self.drape = Some(key);
        Ok(changed)
    }

    /// Forget the last drape, so the next regeneration drapes again.
    pub(super) fn forget_drape(&mut self) {
        self.drape = None;
    }
}

/// What a piece's fit depends on besides the body.
fn fit_key(piece: &FittedPiece<'_>) -> Result<Key> {
    Ok(serde_json::to_vec(&(
        &piece.piece.item.id,
        &piece.piece.placement.id,
        &piece.design,
    ))?)
}

/// What a piece's finish depends on besides its fit.
fn finish_key(fit: &Key, piece: &FittedPiece<'_>) -> Result<Key> {
    Ok(serde_json::to_vec(&(
        fit,
        &piece.decoration,
        &piece.construction,
    ))?)
}
