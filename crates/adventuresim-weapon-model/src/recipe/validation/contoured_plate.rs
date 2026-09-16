//! Input bounds for dimensioned forged blanks.
use super::*;

const MAX_PLATE_SPANS: usize = 64;
const MAX_PLATE_THICKNESS_STATIONS: usize = 16;

pub(super) fn validate(p: &ContouredPlateParameters) -> Checked {
    positive(p.width.get())?;
    positive(p.length.get())?;
    require(
        (3..=MAX_PLATE_SPANS).contains(&p.boundary.len()),
        RecipeError::Budget,
    )?;
    require(
        (2..=MAX_PLATE_THICKNESS_STATIONS).contains(&p.thickness.len()),
        RecipeError::Budget,
    )?;
    let check_point = |point: [Ratio; 2]| {
        require(
            (-1.0..=1.0).contains(&point[0].get()) && (0.0..=1.0).contains(&point[1].get()),
            RecipeError::Profile,
        )
    };
    check_point(p.start)?;
    for span in &p.boundary {
        check_point(span.end())?;
        if let PlateBoundarySpan::Cubic { controls, .. } = span {
            for &point in controls {
                check_point(point)?;
            }
        }
    }
    require(
        p.boundary.last().unwrap().end() == p.start,
        RecipeError::Profile,
    )?;
    require(
        p.thickness.first().unwrap().at.get() == 0.0 && p.thickness.last().unwrap().at.get() == 1.0,
        RecipeError::Profile,
    )?;
    for endpoint in [0.0, 1.0] {
        require(
            std::iter::once(p.start)
                .chain(p.boundary.iter().map(PlateBoundarySpan::end))
                .any(|point| point[1].get() == endpoint),
            RecipeError::Profile,
        )?;
    }
    for pair in p.thickness.windows(2) {
        require(pair[0].at.get() < pair[1].at.get(), RecipeError::Profile)?;
    }
    for station in &p.thickness {
        require(
            (0.0..=1.0).contains(&station.at.get()),
            RecipeError::Profile,
        )?;
        nonnegative(station.flat_half_width.get())?;
        nonnegative(station.hollow_depth())?;
        require(
            station.hollow_depth() <= (station.ridge.get() - station.edge.get()) / 4.0,
            RecipeError::Profile,
        )?;
        let terminal = station.at.get() == 0.0 || station.at.get() == 1.0;
        if terminal && station.edge.get() == 0.0 && station.ridge.get() == 0.0 {
            require(
                station.ridge_half_width.get() == 0.0 && station.flat_half_width.get() == 0.0,
                RecipeError::Profile,
            )?;
            let points = std::iter::once(p.start).chain(
                p.boundary
                    .iter()
                    .take(p.boundary.len() - 1)
                    .map(PlateBoundarySpan::end),
            );
            require(
                points.filter(|point| point[1] == station.at).count() == 1,
                RecipeError::Profile,
            )?;
        } else {
            positive(station.ridge_half_width.get())?;
            require(
                station.ridge_half_width.get() - station.flat_half_width.get()
                    >= MIN_MANUFACTURED_METRES,
                RecipeError::Profile,
            )?;
            positive(station.edge.get())?;
            positive(station.ridge.get())?;
            require(
                station.ridge.get() >= station.edge.get(),
                RecipeError::Profile,
            )?;
        }
    }
    Ok(())
}
