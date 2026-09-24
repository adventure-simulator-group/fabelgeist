//! Names the host and the helmet kernel share: the design float slots and
//! the kinds of carrier vertex. Each is declared once here, and the WGSL
//! constants are generated from the declaration.

/// Declares a fieldless enum and the WGSL `u32` constants naming its
/// variants' discriminants, in `SCREAMING_SNAKE_CASE` with a prefix.
macro_rules! shared_codes {
    ($(#[$meta:meta])* $name:ident { $($(#[$variant_meta:meta])* $variant:ident),* $(,)? }) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub(super) enum $name {
            $($(#[$variant_meta])* $variant),*
        }

        impl $name {
            #[allow(dead_code, reason = "not every code set is counted")]
            pub(super) const COUNT: usize = [$(stringify!($variant)),*].len();

            /// WGSL constants for every variant.
            pub(super) fn wgsl(prefix: &str) -> String {
                [$((stringify!($variant), $name::$variant as u32)),*]
                    .iter()
                    .map(|(variant, code)| {
                        format!("const {prefix}{}: u32 = {code}u;\n", screaming(variant))
                    })
                    .collect()
            }
        }
    };
}

fn screaming(camel: &str) -> String {
    let mut name = String::new();
    for (i, c) in camel.chars().enumerate() {
        if c.is_ascii_uppercase() && i > 0 {
            name.push('_');
        }
        name.push(c.to_ascii_uppercase());
    }
    name
}

shared_codes! {
    /// A helmet's design floats, by slot. The bowl remap table follows them.
    Slot {
        /// Lining clearance plus wall: from the bare head to the carrier.
        Gap,
        CrownHeight,
        /// The brow's rise above the frame's brow line, per unit head height.
        BrowRise,
        /// Added to the skull's fore-aft radius.
        ZExtension,
        /// The dome's latitude exponent.
        Fullness,
        Crest,
        Ridge,
        /// Which angular remap follows the dome: a [`PostMap`] code.
        PostMap,
        Fluted,
        FluteCount,
        FluteWidth,
        FluteDepth,
        FluteSpread,
        FluteLowerSpread,
        FluteStart,
        FluteEnd,
        FluteFade,
        BrimWidth,
        BrimSweep,
        BrimDrop,
        FrontReach,
        BackReach,
        BackSweep,
        EyeOpening,
        MouthOpening,
        EyeHeight,
        OpeningRoundness,
        RearEdgeLift,
        NapeFlare,
        ChinTaper,
        CheekDepth,
        Gauge,
        NapeDepth,
        NapeTaper,
        NapeRecession,
        NeckFlare,
        PeakLength,
        PeakDrop,
        CheekWidth,
        CheekTaper,
        OpeningWidth,
        TailWidth,
        TailLength,
        TailDrop,
        SightGap,
        PivotRise,
        VisorHeight,
        SidePanel,
        VisorProjection,
        VisorSpacing,
    }
}

shared_codes! {
    /// What a carrier vertex is, as its first coordinate tells the kernel.
    VertexKind {
        DomePole,
        /// Latitude, meridian angle.
        DomeRing,
        /// Meridian angle.
        FanRim,
        /// Latitude, across-crown coordinate.
        FanPoint,
        /// Side of the meridian, whether the crest base, across-crown
        /// coordinate.
        FanCrest,
        /// Brim fraction, meridian angle.
        Brim,
        /// Row selector, row step, meridian angle.
        BarbuteCheek,
        /// Nape fraction, skirt fraction, meridian angle.
        Nape,
        /// Peak fraction, meridian angle.
        Peak,
        /// Nape fraction, meridian angle.
        Guard,
        /// Skirt fraction, meridian angle.
        SalletSkirt,
        /// Visor fraction, meridian angle.
        Visor,
    }
}

shared_codes! {
    /// The angular remap a helmet applies to its dome.
    PostMap {
        None,
        /// The barbute's bowl, resampled through its angle table.
        BowlTable,
        /// The sallet's front arc, stretched to its opening.
        FrontArc,
    }
}

shared_codes! {
    /// Rows of a barbute's cheeks.
    CheekRow {
        Shelf,
        Corner,
        Lower,
    }
}
