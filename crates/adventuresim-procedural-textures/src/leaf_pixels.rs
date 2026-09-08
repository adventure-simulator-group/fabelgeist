pub(crate) struct LeafPixels {
    pub opacity: Vec<u8>,
    pub front: Vec<u8>,
    pub back: Vec<u8>,
    pub normal_front: Vec<u8>,
    pub normal_back: Vec<u8>,
    pub height_map: Vec<u8>,
    pub arm: Vec<u8>,
}
impl LeafPixels {
    pub fn upload(
        self,
        params: &crate::TextureParameters,
        images: &mut bevy::asset::Assets<bevy::image::Image>,
    ) -> crate::LeafTextureSet {
        let Self {
            opacity,
            front,
            back,
            normal_front,
            normal_back,
            height_map,
            arm,
        } = self;
        use crate::foliage::{LeafMipSemantic, leaf_mipped_image};
        crate::LeafTextureSet {
            opacity: images.add(leaf_mipped_image(
                params,
                opacity,
                false,
                LeafMipSemantic::Coverage,
            )),
            front_albedo: images.add(leaf_mipped_image(
                params,
                front,
                true,
                LeafMipSemantic::ColorCoverage,
            )),
            back_albedo: images.add(leaf_mipped_image(
                params,
                back,
                true,
                LeafMipSemantic::ColorCoverage,
            )),
            front_normal: images.add(leaf_mipped_image(
                params,
                normal_front,
                false,
                LeafMipSemantic::Normal,
            )),
            back_normal: images.add(leaf_mipped_image(
                params,
                normal_back,
                false,
                LeafMipSemantic::Normal,
            )),
            height: images.add(leaf_mipped_image(
                params,
                height_map,
                false,
                LeafMipSemantic::Scalar,
            )),
            arm: images.add(leaf_mipped_image(
                params,
                arm,
                false,
                LeafMipSemantic::Scalar,
            )),
        }
    }
}
