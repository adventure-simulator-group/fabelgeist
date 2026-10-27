//! Skin weights of a vertex blended from others.

/// The joints a vertex follows and how much, heaviest first.
pub(crate) type Skin = ([u32; 8], [f32; 8]);

/// The heaviest eight joints of `skins`, each weighted by its share,
/// renormalised. With no weight at all, the first skin.
pub(crate) fn blend(skins: &[(Skin, f32)]) -> Skin {
    let mut merged = Vec::<(u32, f32)>::with_capacity(skins.len() * 8);
    for ((joints, weights), share) in skins {
        for (joint, weight) in joints.iter().zip(weights) {
            let weight = weight * share;
            if weight <= 0.0 {
                continue;
            }
            match merged.iter_mut().find(|(j, _)| j == joint) {
                Some((_, total)) => *total += weight,
                None => merged.push((*joint, weight)),
            }
        }
    }
    merged.sort_by(|x, y| y.1.total_cmp(&x.1).then(x.0.cmp(&y.0)));
    merged.truncate(8);
    let total = merged.iter().map(|(_, w)| w).sum::<f32>();
    if total <= 0.0 {
        return skins[0].0;
    }
    let mut joints = [0u32; 8];
    let mut weights = [0.0f32; 8];
    for (slot, (joint, weight)) in merged.into_iter().enumerate() {
        joints[slot] = joint;
        weights[slot] = weight / total;
    }
    (joints, weights)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn single(joint: u32) -> Skin {
        let mut joints = [0; 8];
        let mut weights = [0.0; 8];
        joints[0] = joint;
        weights[0] = 1.0;
        (joints, weights)
    }

    #[test]
    fn shares_split_the_weight_heaviest_first() {
        let (joints, weights) = blend(&[(single(3), 0.25), (single(7), 0.75)]);
        assert_eq!(&joints[..2], &[7, 3]);
        assert_eq!(&weights[..2], &[0.75, 0.25]);
    }

    #[test]
    fn no_weight_keeps_the_first_skin() {
        assert_eq!(blend(&[(single(3), 0.0), (single(7), 0.0)]), single(3));
    }
}
