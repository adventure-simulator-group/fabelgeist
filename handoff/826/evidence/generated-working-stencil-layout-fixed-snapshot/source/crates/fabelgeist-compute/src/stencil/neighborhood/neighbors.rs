//! Adjacent sample loads and the source protocol's directional aliases.

use super::fetch::NeighborhoodFetch;
use super::{NeighborhoodDimension, NeighborhoodParameter, sanitize_type_name};

pub(super) fn emit(
    code: &mut String,
    neighborhood: &NeighborhoodParameter,
    fetch: &NeighborhoodFetch,
) {
    let type_name = sanitize_type_name(&neighborhood.element_type().as_str());
    match neighborhood.dimension() {
        NeighborhoodDimension::One => one(code, fetch, &type_name),
        NeighborhoodDimension::Two => two(code, fetch, &type_name),
        NeighborhoodDimension::Three => three(code, fetch, &type_name),
        NeighborhoodDimension::Four => four(code, fetch, &type_name),
    }
}

fn one(code: &mut String, fetch: &NeighborhoodFetch, type_name: &str) {
    code.push_str(&format!("    let center = {};\n", fetch.at("coords")));

    code.push_str(&format!(
        "    let _n_left = {};\n",
        fetch.at("get_coord_1d(coords - 1, _grid_w)")
    ));
    code.push_str(&format!(
        "    let _n_right = {};\n",
        fetch.at("get_coord_1d(coords + 1, _grid_w)")
    ));
    code.push_str("    let _n_prev = _n_left;\n");
    code.push_str("    let _n_next = _n_right;\n");
    code.push_str(&format!(
        "    let neighbors = Neighbors1D_{}(_n_left, _n_right, _n_prev, _n_next);\n",
        type_name
    ));
}

fn two(code: &mut String, fetch: &NeighborhoodFetch, type_name: &str) {
    code.push_str(&format!("    let center = {};\n", fetch.at("coords")));

    code.push_str(&format!(
        "    let _n_left = {};\n",
        fetch.at("get_coord_2d(coords + vec2<i32>(-1, 0), _grid_dim)")
    ));
    code.push_str(&format!(
        "    let _n_right = {};\n",
        fetch.at("get_coord_2d(coords + vec2<i32>(1, 0), _grid_dim)")
    ));
    code.push_str(&format!(
        "    let _n_top = {};\n",
        fetch.at("get_coord_2d(coords + vec2<i32>(0, -1), _grid_dim)")
    ));
    code.push_str(&format!(
        "    let _n_bottom = {};\n",
        fetch.at("get_coord_2d(coords + vec2<i32>(0, 1), _grid_dim)")
    ));
    code.push_str("    let _n_up = _n_top;\n");
    code.push_str("    let _n_down = _n_bottom;\n");
    code.push_str(&format!(
                        "    let neighbors = Neighbors2D_{}(_n_left, _n_right, _n_top, _n_bottom, _n_up, _n_down);\n",
                        type_name
                    ));
}

fn three(code: &mut String, fetch: &NeighborhoodFetch, type_name: &str) {
    code.push_str(&format!("    let center = {};\n", fetch.at("coords")));

    code.push_str(&format!(
        "    let _n_left = {};\n",
        fetch.at("get_coord_3d(coords + vec3<i32>(-1, 0, 0), _grid_dim)")
    ));
    code.push_str(&format!(
        "    let _n_right = {};\n",
        fetch.at("get_coord_3d(coords + vec3<i32>(1, 0, 0), _grid_dim)")
    ));
    code.push_str(&format!(
        "    let _n_top = {};\n",
        fetch.at("get_coord_3d(coords + vec3<i32>(0, -1, 0), _grid_dim)")
    ));
    code.push_str(&format!(
        "    let _n_bottom = {};\n",
        fetch.at("get_coord_3d(coords + vec3<i32>(0, 1, 0), _grid_dim)")
    ));
    code.push_str("    let _n_up = _n_top;\n");
    code.push_str("    let _n_down = _n_bottom;\n");
    code.push_str(&format!(
        "    let _n_front = {};\n",
        fetch.at("get_coord_3d(coords + vec3<i32>(0, 0, -1), _grid_dim)")
    ));
    code.push_str(&format!(
        "    let _n_back = {};\n",
        fetch.at("get_coord_3d(coords + vec3<i32>(0, 0, 1), _grid_dim)")
    ));
    code.push_str(&format!(
                        "    let neighbors = Neighbors3D_{}(_n_left, _n_right, _n_top, _n_bottom, _n_up, _n_down, _n_front, _n_back);\n",
                        type_name
                    ));
}

fn four(code: &mut String, fetch: &NeighborhoodFetch, type_name: &str) {
    code.push_str(&format!("    let center = {};\n", fetch.at("coords")));

    code.push_str(&format!(
        "    let _n_left = {};\n",
        fetch.at("get_coord_4d(coords + vec4<i32>(-1, 0, 0, 0), _grid_dim)")
    ));
    code.push_str(&format!(
        "    let _n_right = {};\n",
        fetch.at("get_coord_4d(coords + vec4<i32>(1, 0, 0, 0), _grid_dim)")
    ));
    code.push_str(&format!(
        "    let _n_top = {};\n",
        fetch.at("get_coord_4d(coords + vec4<i32>(0, -1, 0, 0), _grid_dim)")
    ));
    code.push_str(&format!(
        "    let _n_bottom = {};\n",
        fetch.at("get_coord_4d(coords + vec4<i32>(0, 1, 0, 0), _grid_dim)")
    ));
    code.push_str("    let _n_up = _n_top;\n");
    code.push_str("    let _n_down = _n_bottom;\n");
    code.push_str(&format!(
        "    let _n_front = {};\n",
        fetch.at("get_coord_4d(coords + vec4<i32>(0, 0, -1, 0), _grid_dim)")
    ));
    code.push_str(&format!(
        "    let _n_back = {};\n",
        fetch.at("get_coord_4d(coords + vec4<i32>(0, 0, 1, 0), _grid_dim)")
    ));
    code.push_str(&format!(
        "    let _n_ana = {};\n",
        fetch.at("get_coord_4d(coords + vec4<i32>(0, 0, 0, -1), _grid_dim)")
    ));
    code.push_str(&format!(
        "    let _n_kata = {};\n",
        fetch.at("get_coord_4d(coords + vec4<i32>(0, 0, 0, 1), _grid_dim)")
    ));
    code.push_str("    let _n_w_left = _n_ana;\n");
    code.push_str("    let _n_w_right = _n_kata;\n");
    code.push_str("    let _n_past = _n_ana;\n");
    code.push_str("    let _n_future = _n_kata;\n");
    code.push_str(&format!(
                        "    let neighbors = Neighbors4D_{}(_n_left, _n_right, _n_top, _n_bottom, _n_up, _n_down, _n_front, _n_back, _n_ana, _n_kata, _n_w_left, _n_w_right, _n_past, _n_future);\n",
                        type_name
                    ));
}
