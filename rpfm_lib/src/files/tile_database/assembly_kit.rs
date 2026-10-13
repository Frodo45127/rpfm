//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Building a tile database from its sources in the Assembly Kit, as BOB's "Process tile database" does.
//!
//! The sources are `raw_data/terrain/tiles/<battle|campaign>/_tile_database/_settings.xml` and one XML per tile in its `TILES` folder.
//! The result is byte for byte what BOB writes from the same sources.

use std::collections::{BTreeMap, HashSet};

use crate::error::{Result, RLibError};
use crate::xml::{XmlElement, parse_elements};

use super::*;
use super::texture::Texture;
use super::tile_link::TileLink;
use super::tile_link_target::TileLinkTarget;
use super::tile_variation::TileVariation;

/// Version BOB writes the render parameters with, whatever the XML says.
const RENDER_PARAMS_VERSION: u16 = 6;

/// Tile set the battle database always ends with.
const USER_TILE_SET: &str = "user";

/// The elements of an XML document, to read attributes and children from.
struct Document {
    elements: Vec<XmlElement>,
    root: usize,
}

//---------------------------------------------------------------------------//
//                     Implementation of TileDatabase
//---------------------------------------------------------------------------//

impl TileDatabase {

    /// Builds a tile database from its sources in the Assembly Kit.
    ///
    /// Tile sets that tiles use but `_settings.xml` doesn't define are added after the defined ones, with empty values,
    /// and so is the `user` set if requested: the battle database always has it, the campaign one doesn't.
    ///
    /// # Arguments
    ///
    /// * `settings` - The contents of `_settings.xml`.
    /// * `tiles` - The contents of each tile's XML.
    /// * `order` - Locations of the tiles, in the order they go in the database. If None, larger tiles go first, then by location.
    /// * `add_user_tile_set` - If the `user` tile set is added.
    ///
    /// # Returns
    ///
    /// The tile database.
    ///
    /// # Errors
    ///
    /// Returns [`RLibError::DecodingTileDatabaseXml`] if any source is malformed, or a tile is missing from `order`.
    pub fn from_assembly_kit(settings: &str, tiles: &[&str], order: Option<&[String]>, add_user_tile_set: bool) -> Result<Self> {
        let settings = Document::parse(settings)?;
        let root = settings.root();
        let mut database = Self::default();
        database.set_serialise_version(1);
        database.set_render_params(settings.render_params(settings.child(root, "RENDER_PARAMS")?)?);
        database.set_conversion_params(settings.conversion_params(settings.child(root, "CONVERSION_PARAMS")?)?);

        let climates = settings.children(settings.child(root, "CLIMATES")?);
        database.set_climates(climates.into_iter().map(|climate| settings.climate(climate)).collect::<Result<_>>()?);

        let tiles = tiles.iter().map(|tile| Document::parse(tile)).collect::<Result<Vec<_>>>()?;

        let mut tile_sets = settings.children(settings.child(root, "TILE_SETS")?).into_iter()
            .map(|tile_set| settings.tile_set(tile_set))
            .collect::<Result<Vec<_>>>()?;

        let mut known = tile_sets.iter().map(|tile_set| tile_set.name().to_owned()).collect::<HashSet<_>>();
        let mut undefined = vec![];
        for tile in &tiles {
            let name = tile.attribute(tile.root(), "tile_set")?;
            if known.insert(name.to_owned()) {
                undefined.push(name.to_owned());
            }
        }

        if add_user_tile_set && known.insert(USER_TILE_SET.to_owned()) {
            undefined.push(USER_TILE_SET.to_owned());
        }

        for name in undefined {
            let mut tile_set = TileSet::default();
            tile_set.set_serialise_version(1);
            tile_set.set_name(name);
            tile_sets.push(tile_set);
        }

        database.set_tile_sets(tile_sets);

        let mut tiles = tiles.iter().map(|tile| tile.tile()).collect::<Result<Vec<_>>>()?;
        match order {
            Some(order) => {
                let rank = order.iter().enumerate().map(|(index, location)| (location.as_str(), index)).collect::<BTreeMap<_, _>>();
                if let Some(tile) = tiles.iter().find(|tile| !rank.contains_key(tile.location().as_str())) {
                    return Err(xml_error(format!("tile missing from the order list: {}", tile.location())));
                }

                tiles.sort_by_key(|tile| rank[tile.location().as_str()]);
            }
            None => tiles.sort_by(|a, b| (b.width() * b.height()).cmp(&(a.width() * a.height())).then_with(|| a.location().cmp(b.location()))),
        }

        database.set_tiles(tiles);
        Ok(database)
    }
}

impl Document {

    fn parse(xml: &str) -> Result<Self> {
        let (elements, roots) = parse_elements(xml).map_err(xml_error)?;
        let root = *roots.first().ok_or_else(|| xml_error("empty document".to_owned()))?;

        Ok(Self { elements, root })
    }

    fn root(&self) -> usize {
        self.root
    }

    fn attribute(&self, element: usize, name: &str) -> Result<&str> {
        let element = &self.elements[element];
        element.attributes.iter()
            .find(|(attribute, _)| attribute == name)
            .map(|(_, value)| value.as_str())
            .ok_or_else(|| xml_error(format!("missing attribute '{name}' on <{}>", element.name)))
    }

    fn f32(&self, element: usize, name: &str) -> Result<f32> {
        let value = self.attribute(element, name)?;
        value.parse().map_err(|_| xml_error(format!("'{value}' of '{name}' is not a number")))
    }

    fn u32(&self, element: usize, name: &str) -> Result<u32> {
        let value = self.attribute(element, name)?;
        value.parse().map_err(|_| xml_error(format!("'{value}' of '{name}' is not an unsigned integer")))
    }

    fn i32(&self, element: usize, name: &str) -> Result<i32> {
        let value = self.attribute(element, name)?;
        value.parse().map_err(|_| xml_error(format!("'{value}' of '{name}' is not an integer")))
    }

    fn u16(&self, element: usize, name: &str) -> Result<u16> {
        let value = self.attribute(element, name)?;
        value.parse().map_err(|_| xml_error(format!("'{value}' of '{name}' is not a version")))
    }

    fn bool(&self, element: usize, name: &str) -> Result<bool> {
        Ok(self.attribute(element, name)? == "true")
    }

    fn string(&self, element: usize, name: &str) -> Result<String> {
        self.attribute(element, name).map(|value| value.to_owned())
    }

    fn child(&self, element: usize, name: &str) -> Result<usize> {
        self.elements[element].children.iter().copied()
            .find(|&child| self.elements[child].name == name)
            .ok_or_else(|| xml_error(format!("missing <{name}> in <{}>", self.elements[element].name)))
    }

    fn children(&self, element: usize) -> Vec<usize> {
        self.elements[element].children.clone()
    }

    fn render_params(&self, element: usize) -> Result<RenderParams> {
        let mut params = RenderParams::default();
        params.set_serialise_version(RENDER_PARAMS_VERSION);
        params.set_lf_height(self.f32(element, "lf_height")?);
        params.set_hf_height(self.f32(element, "hf_height")?);
        params.set_blend_pixel_scale(self.f32(element, "blend_pixel_scale")?);
        params.set_unit_scale(self.f32(element, "unit_scale")?);
        params.set_mid_distance_detail_scale(self.f32(element, "mid_distance_detail_scale")?);
        params.set_mid_distance_detail_strength(self.f32(element, "mid_distance_detail_strength")?);
        params.set_mid_distance_normal_strength(self.f32(element, "mid_distance_normal_strength")?);
        params.set_mid_distance_detail_near(self.f32(element, "mid_distance_detail_near")?);
        params.set_mid_distance_detail_far(self.f32(element, "mid_distance_detail_far")?);
        params.set_mid_distance_detail_slope_low(self.f32(element, "mid_distance_detail_slope_low")?);
        params.set_mid_distance_detail_slope_high(self.f32(element, "mid_distance_detail_slope_high")?);
        params.set_vertical_offset(self.f32(element, "vertical_offset")?);
        params.set_normal_lf_scale(self.f32(element, "normal_lf_scale")?);
        params.set_normal_tile_scale(self.f32(element, "normal_tile_scale")?);
        params.set_normal_terrain_scale(self.f32(element, "normal_terrain_scale")?);
        params.set_blend_contrast(self.f32(element, "blend_contrast")?);
        params.set_layer_exempt_0(self.string(element, "layer_exempt_0")?);
        params.set_layer_exempt_1(self.string(element, "layer_exempt_1")?);
        params.set_layer_exempt_2(self.string(element, "layer_exempt_2")?);
        params.set_layer_exempt_3(self.string(element, "layer_exempt_3")?);
        params.set_campaign_sea_transparency_scale(self.f32(element, "campaign_sea_transparency_scale")?);
        params.set_campaign_sea_uv_scale(self.f32(element, "campaign_sea_uv_scale")?);
        params.set_campaign_lake_transparency_scale(self.f32(element, "campaign_lake_transparency_scale")?);
        params.set_near_distance_detail_distance(self.f32(element, "near_distance_detail_distance")?);
        params.set_near_distance_detail_scale(self.f32(element, "near_distance_detail_scale")?);
        params.set_near_distance_detail_strength(self.f32(element, "near_distance_detail_strength")?);
        params.set_mid_distance_detail_near_lq(self.f32(element, "mid_distance_detail_near_lq")?);
        Ok(params)
    }

    fn conversion_params(&self, element: usize) -> Result<ConversionParams> {
        let mut params = ConversionParams::default();
        params.set_serialise_version(self.u16(element, "serialise_version")?);
        params.set_triangle_density(self.i32(element, "triangle_density")?);
        params.set_max_lf_heights_per_pixel(self.i32(element, "max_lf_heights_per_pixel")?);
        params.set_triangle_decimation_size_factors0(self.f32(element, "triangle_decimation_size_factors0")?);
        params.set_triangle_decimation_size_factors1(self.f32(element, "triangle_decimation_size_factors1")?);
        params.set_triangle_decimation_size_factors2(self.f32(element, "triangle_decimation_size_factors2")?);
        params.set_triangle_decimation_size_factors3(self.f32(element, "triangle_decimation_size_factors3")?);
        params.set_triangle_decimation_size_factors4(self.f32(element, "triangle_decimation_size_factors4")?);
        params.set_triangle_decimation_size_factors5(self.f32(element, "triangle_decimation_size_factors5")?);
        params.set_triangle_decimation_angle_factors0(self.f32(element, "triangle_decimation_angle_factors0")?);
        params.set_triangle_decimation_angle_factors1(self.f32(element, "triangle_decimation_angle_factors1")?);
        params.set_triangle_decimation_angle_factors2(self.f32(element, "triangle_decimation_angle_factors2")?);
        params.set_triangle_decimation_angle_factors3(self.f32(element, "triangle_decimation_angle_factors3")?);
        params.set_triangle_decimation_angle_factors4(self.f32(element, "triangle_decimation_angle_factors4")?);
        params.set_triangle_decimation_angle_factors5(self.f32(element, "triangle_decimation_angle_factors5")?);
        Ok(params)
    }

    fn climate(&self, element: usize) -> Result<Climate> {
        let mut climate = Climate::default();
        climate.set_serialise_version(self.u16(element, "serialise_version")?);
        climate.set_name(self.string(element, "name")?);
        climate.set_texture_set(self.string(element, "texture_set")?);
        climate.set_red(self.f32(element, "red")?);
        climate.set_green(self.f32(element, "green")?);
        climate.set_blue(self.f32(element, "blue")?);
        climate.set_textures(self.children(self.child(element, "TEXTURES")?).into_iter().map(|texture| self.texture(texture)).collect::<Result<_>>()?);
        climate.set_grass_alpha_add(self.f32(element, "grass_alpha_add")?);
        climate.set_grass_alpha_mul(self.f32(element, "grass_alpha_mul")?);
        climate.set_destruction_climate(self.string(element, "destruction_climate")?);
        Ok(climate)
    }

    fn texture(&self, element: usize) -> Result<Texture> {
        let mut texture = Texture::default();
        texture.set_serialise_version(self.u16(element, "serialise_version")?);
        texture.set_name(self.string(element, "name")?);
        texture.set_mid_distance_strength(self.f32(element, "mid_distance_strength")?);
        texture.set_lerp_reflectivity0(self.f32(element, "lerp_reflectivity0")?);
        texture.set_lerp_smoothness0(self.f32(element, "lerp_smoothness0")?);
        texture.set_lerp_reflectivity1(self.f32(element, "lerp_reflectivity1")?);
        texture.set_lerp_smoothness1(self.f32(element, "lerp_smoothness1")?);
        texture.set_lerp_reflectivity2(self.f32(element, "lerp_reflectivity2")?);
        texture.set_lerp_smoothness2(self.f32(element, "lerp_smoothness2")?);
        texture.set_lerp_reflectivity3(self.f32(element, "lerp_reflectivity3")?);
        texture.set_lerp_smoothness3(self.f32(element, "lerp_smoothness3")?);
        texture.set_blend_pixel_scale(self.f32(element, "blend_pixel_scale")?);
        texture.set_outfield_blend_pixel_scale(self.f32(element, "outfield_blend_pixel_scale")?);
        Ok(texture)
    }

    fn tile_set(&self, element: usize) -> Result<TileSet> {
        let mut tile_set = TileSet::default();
        tile_set.set_serialise_version(self.u16(element, "serialise_version")?);
        tile_set.set_name(self.string(element, "name")?);
        tile_set.set_linking_tile(self.string(element, "linking_tile")?);
        tile_set.set_shared_geometry(self.string(element, "shared_geometry")?);
        tile_set.set_also_place_tile_set(self.string(element, "also_place_tile_set")?);
        tile_set.set_link_as_set(self.string(element, "link_as_set")?);
        tile_set.set_red(self.f32(element, "red")?);
        tile_set.set_green(self.f32(element, "green")?);
        tile_set.set_blue(self.f32(element, "blue")?);
        Ok(tile_set)
    }

    fn tile(&self) -> Result<Tile> {
        let element = self.root;
        let mut tile = Tile::default();
        tile.set_serialise_version(self.u16(element, "serialise_version")?);
        tile.set_location(self.string(element, "location")?);
        tile.set_tile_set(self.string(element, "tile_set")?);
        tile.set_mask(self.string(element, "mask")?);
        tile.set_width(self.u32(element, "width")?);
        tile.set_height(self.u32(element, "height")?);
        tile.set_red(self.f32(element, "red")?);
        tile.set_green(self.f32(element, "green")?);
        tile.set_blue(self.f32(element, "blue")?);
        tile.set_requires_infield_lodding(self.bool(element, "requires_infield_lodding")?);
        tile.set_random_rotatable(self.bool(element, "random_rotatable")?);
        tile.set_custom_alpha_blend_texture(self.string(element, "custom_alpha_blend_texture")?);
        tile.set_scalable(self.bool(element, "scalable")?);
        tile.set_encampable(self.bool(element, "encampable")?);
        tile.set_custom_blend_tile(self.string(element, "custom_blend_tile")?);

        let texture_group = self.child(element, "TEXTURE_GROUP")?;
        tile.set_texture_red(self.string(texture_group, "red")?);
        tile.set_texture_green(self.string(texture_group, "green")?);
        tile.set_texture_blue(self.string(texture_group, "blue")?);
        tile.set_texture_alpha(self.string(texture_group, "alpha")?);

        tile.set_variations(self.children(self.child(element, "VARIATIONS")?).into_iter().map(|variation| self.tile_variation(variation)).collect::<Result<_>>()?);
        tile.set_link_targets(self.children(self.child(element, "TILE_LINK_TARGETS")?).into_iter().map(|target| self.tile_link_target(target)).collect::<Result<_>>()?);
        tile.set_links(self.children(self.child(element, "TILE_LINKS")?).into_iter().map(|link| self.tile_link(link)).collect::<Result<_>>()?);
        Ok(tile)
    }

    fn tile_variation(&self, element: usize) -> Result<TileVariation> {
        let mut variation = TileVariation::default();
        variation.set_serialise_version(self.u16(element, "serialise_version")?);
        variation.set_location(self.string(element, "location")?);
        variation.set_min_height(self.f32(element, "min_height")?);
        variation.set_scale(self.f32(element, "scale")?);
        variation.set_normal_strength(self.f32(element, "normal_strength")?);
        variation.set_overlap_border_size(self.f32(element, "overlap_border_size")?);
        variation.set_raw_data_tri_density(self.u32(element, "raw_data_tri_density")?);
        variation.set_blend_common(self.string(element, "blend_common")?);
        variation.set_index_common(self.string(element, "index_common")?);
        variation.set_normal_common(self.string(element, "normal_common")?);
        variation.set_red(self.f32(element, "red")?);
        variation.set_green(self.f32(element, "green")?);
        variation.set_blue(self.f32(element, "blue")?);
        variation.set_barbarian(self.bool(element, "barbarian")?);
        Ok(variation)
    }

    fn tile_link_target(&self, element: usize) -> Result<TileLinkTarget> {
        let mut target = TileLinkTarget::default();
        target.set_serialise_version(self.u16(element, "serialise_version")?);
        target.set_tile_set(self.string(element, "tile_set")?);
        target.set_x(self.i32(element, "x")?);
        target.set_y(self.i32(element, "y")?);
        Ok(target)
    }

    fn tile_link(&self, element: usize) -> Result<TileLink> {
        let mut link = TileLink::default();
        link.set_serialise_version(self.u16(element, "serialise_version")?);
        link.set_link_set(self.string(element, "link_set")?);
        link.set_x(self.i32(element, "x")?);
        link.set_y(self.i32(element, "y")?);
        link.set_base_x(self.i32(element, "base_x")?);
        link.set_base_y(self.i32(element, "base_y")?);
        link.set_is_entry(self.bool(element, "is_entry")?);
        link.set_blend_size(self.u32(element, "blend_size")?);
        link.set_no_offline_blend(self.bool(element, "no_offline_blend")?);
        link.set_test(self.string(element, "test")?);
        Ok(link)
    }
}

/// Error of a source that can't be read.
fn xml_error(reason: String) -> RLibError {
    RLibError::DecodingTileDatabaseXml(reason)
}
