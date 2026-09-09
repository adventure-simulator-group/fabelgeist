use super::*;

impl ShopSignRenderCache {
    pub(super) fn painted_material(
        &mut self,
        sign: &ShopSign,
        board: SignBoard,
        detail: SignDetail,
        materials: &mut Assets<StandardMaterial>,
        images: &mut Assets<Image>,
    ) -> Option<Handle<StandardMaterial>> {
        if detail == SignDetail::Board {
            return None;
        }
        let key = (
            sign.name.clone(),
            sign.font,
            sign.finish,
            (board.size.y / board.size.x * crate::signs::lettering::TEXTURE_WIDTH as f32).round()
                as u32,
        );
        if self.paint.len() >= MAX_CACHED_SIGN_TEXTURES && !self.paint.contains_key(&key) {
            self.paint.clear();
        }
        Some(
            self.paint
                .entry(key)
                .or_insert_with(|| {
                    let painted = SignTexture::paint(sign, board.size);
                    let image = Image::new(
                        Extent3d {
                            width: painted.width,
                            height: painted.height,
                            depth_or_array_layers: 1,
                        },
                        TextureDimension::D2,
                        painted.rgba,
                        TextureFormat::Rgba8UnormSrgb,
                        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
                    );
                    materials.add(StandardMaterial {
                        base_color_texture: Some(images.add(image)),
                        perceptual_roughness: 0.92,
                        ..default()
                    })
                })
                .clone(),
        )
    }
    pub(super) fn backing_material(
        &mut self,
        finish: SignFinish,
        materials: &mut Assets<StandardMaterial>,
    ) -> Handle<StandardMaterial> {
        self.backing
            .entry(finish)
            .or_insert_with(|| {
                let color = finish.colors().0;
                materials.add(StandardMaterial {
                    base_color: Color::srgb_u8(color[0], color[1], color[2]),
                    perceptual_roughness: 0.92,
                    ..default()
                })
            })
            .clone()
    }
    pub(super) fn iron_material(
        &mut self,
        materials: &mut Assets<StandardMaterial>,
    ) -> Handle<StandardMaterial> {
        self.iron
            .get_or_insert_with(|| {
                materials.add(StandardMaterial {
                    base_color: Color::srgb(0.08, 0.07, 0.06),
                    metallic: 0.65,
                    perceptual_roughness: 0.7,
                    ..default()
                })
            })
            .clone()
    }
}
