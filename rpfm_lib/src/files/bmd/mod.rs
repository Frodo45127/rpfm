//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This is a module to read/write Battle Map Definition binary (FASTBIN0) files.

use getset::*;
use nalgebra::{Matrix3, Rotation3};
use serde_derive::{Serialize, Deserialize};

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{bmd_vegetation::BmdVegetation, Decodeable, DecodeableExtraData, Encodeable, EncodeableExtraData};
use crate::utils::check_size_mismatch;

use self::battlefield_building_list::BattlefieldBuildingList;
use self::battlefield_building_list_far::BattlefieldBuildingListFar;
use self::capture_location_set::CaptureLocationSet;
use self::common::*;
use self::ef_line_list::EFLineList;
use self::go_outlines::GoOutlines;
use self::non_terrain_outlines::NonTerrainOutlines;
use self::zones_template_list::ZonesTemplateList;
use self::prefab_instance_list::PrefabInstanceList;
use self::bmd_outline_list::BmdOutlineList;
use self::terrain_outlines::TerrainOutlines;
use self::lite_building_outlines::LiteBuildingOutlines;
use self::camera_zones::CameraZones;
use self::civilian_deployment_list::CivilianDeploymentList;
use self::civilian_shelter_list::CivilianShelterList;
use self::prop_list::PropList;
use self::particle_emitter_list::ParticleEmitterList;
use self::ai_hints::AIHints;
use self::light_probe_list::LightProbeList;
use self::terrain_stencil_triangle_list::TerrainStencilTriangleList;
use self::point_light_list::PointLightList;
use self::building_projectile_emitter_list::BuildingProjectileEmitterList;
use self::playable_area::PlayableArea;
use self::custom_material_mesh_list::CustomMaterialMeshList;
use self::terrain_stencil_blend_triangle_list::TerrainStencilBlendTriangleList;
use self::spot_light_list::SpotLightList;
use self::sound_shape_list::SoundShapeList;
use self::composite_scene_list::CompositeSceneList;
use self::deployment_list::DeploymentList;
use self::bmd_catchment_area_list::BmdCatchmentAreaList;
use self::toggleable_buildings_slot_list::ToggleableBuildingsSlotList;
use self::terrain_decal_list::TerrainDecalList;
use self::tree_list_reference_list::TreeListReferenceList;
use self::grass_list_reference_list::GrassListReferenceList;
use self::water_outlines::WaterOutlines;

/// Extensions used by BMD files.
pub const EXTENSIONS: [&str; 1] = [
    ".bmd",
];

/// FASTBIN0
pub const SIGNATURE: &[u8; 8] = &[0x46, 0x41, 0x53, 0x54, 0x42, 0x49, 0x4E, 0x30];

mod battlefield_building_list;
mod battlefield_building_list_far;
mod capture_location_set;
mod ef_line_list;
mod go_outlines;
mod non_terrain_outlines;
mod zones_template_list;
mod prefab_instance_list;
mod bmd_outline_list;
mod terrain_outlines;
mod lite_building_outlines;
mod camera_zones;
mod civilian_deployment_list;
mod civilian_shelter_list;
mod prop_list;
mod particle_emitter_list;
mod ai_hints;
mod light_probe_list;
mod terrain_stencil_triangle_list;
mod point_light_list;
mod building_projectile_emitter_list;
mod playable_area;
mod custom_material_mesh_list;
mod terrain_stencil_blend_triangle_list;
mod spot_light_list;
mod sound_shape_list;
mod composite_scene_list;
mod deployment_list;
mod bmd_catchment_area_list;
mod toggleable_buildings_slot_list;
mod terrain_decal_list;
mod tree_list_reference_list;
mod grass_list_reference_list;
mod water_outlines;

pub mod common;
mod v23;
mod v24;
mod v25;
mod v26;
mod v27;

#[cfg(test)] mod bmd_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This holds an entire `Bmd` file decoded in memory.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Bmd {
    serialise_version: u16,

    battlefield_building_list: BattlefieldBuildingList,
    battlefield_building_list_far: BattlefieldBuildingListFar,
    capture_location_set: CaptureLocationSet,
    ef_line_list: EFLineList,
    go_outlines: GoOutlines,
    non_terrain_outlines: NonTerrainOutlines,
    zones_template_list: ZonesTemplateList,
    prefab_instance_list: PrefabInstanceList,
    bmd_outline_list: BmdOutlineList,
    terrain_outlines: TerrainOutlines,
    lite_building_outlines: LiteBuildingOutlines,
    camera_zones: CameraZones,
    civilian_deployment_list: CivilianDeploymentList,
    civilian_shelter_list: CivilianShelterList,
    prop_list: PropList,
    particle_emitter_list: ParticleEmitterList,
    ai_hints: AIHints,
    light_probe_list: LightProbeList,
    terrain_stencil_triangle_list: TerrainStencilTriangleList,
    point_light_list: PointLightList,
    building_projectile_emitter_list: BuildingProjectileEmitterList,
    playable_area: PlayableArea,
    custom_material_mesh_list: CustomMaterialMeshList,
    terrain_stencil_blend_triangle_list: TerrainStencilBlendTriangleList,
    spot_light_list: SpotLightList,
    sound_shape_list: SoundShapeList,
    composite_scene_list: CompositeSceneList,
    deployment_list: DeploymentList,
    bmd_catchment_area_list: BmdCatchmentAreaList,
    toggleable_buildings_slot_list: ToggleableBuildingsSlotList,
    terrain_decal_list: TerrainDecalList,
    tree_list_reference_list: TreeListReferenceList,
    grass_list_reference_list: GrassListReferenceList,
    water_outlines: WaterOutlines,
}

//---------------------------------------------------------------------------//
//                           Implementation of Bmd
//---------------------------------------------------------------------------//

pub trait ToLayer {
    fn to_layer(&self, _parent: &Bmd) -> Result<String> {
        Ok(String::new())
    }
}

impl Decodeable for Bmd {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let signature_bytes = data.read_slice(8, false)?;
        if signature_bytes.as_slice() != SIGNATURE {
            return Err(RLibError::DecodingFastBinUnsupportedSignature(signature_bytes));
        }

        let mut fastbin = Self::default();
        fastbin.serialise_version = data.read_u16()?;

        match fastbin.serialise_version {
            23 => fastbin.read_v23(data, extra_data)?,
            24 => fastbin.read_v24(data, extra_data)?,
            25 => fastbin.read_v25(data, extra_data)?,
            26 => fastbin.read_v26(data, extra_data)?,
            27 => fastbin.read_v27(data, extra_data)?,
            _ => return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("Bmd"), fastbin.serialise_version)),
        }

        // If we are not in the last byte, it means we didn't parse the entire file, which means this file is corrupt.
        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;

        Ok(fastbin)
    }
}

impl Encodeable for Bmd {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_all(SIGNATURE)?;
        buffer.write_u16(self.serialise_version)?;

        match self.serialise_version {
            23 => self.write_v23(buffer, extra_data)?,
            24 => self.write_v24(buffer, extra_data)?,
            25 => self.write_v25(buffer, extra_data)?,
            26 => self.write_v26(buffer, extra_data)?,
            27 => self.write_v27(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("Bmd"), self.serialise_version)),
        }

        Ok(())
    }
}

impl Bmd {
    pub fn export_prefab_to_raw_data(&self, name: &str, vegetation: Option<&BmdVegetation>, output_path: &Path) -> Result<()> {

        // We need to generate two files:
        // - .terry: The project file with just one layer.
        // - .layer: The layer file with the contents of the bmd and bmd_vegetation.
        let terry_path = output_path.join(format!("{}.terry", name));
        let layer_path = output_path.join(format!("{}.187abf10b8b9a13.layer", name));

        let terry_data = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<project version=\"27\" id=\"187abf10b7296f5\">
  <pc type=\"QTU::ProjectPrefab\">
    <data database=\"battle\" is_skybox=\"0\"/>
  </pc>
  <pc type=\"QTU::Scene\">
    <data version=\"41\">
      <entity id=\"187abf10b8b9a13\" name=\"Default\">
        <ECFileLayer export=\"true\" bmd_export_type=\"\"/>
        <ECFileLayer export=\"true\" bmd_export_type=\"\"/>
      </entity>
    </data>
  </pc>
  <pc type=\"QTU::Terrain\"/>
</project>".to_string();

        let mut terry_file = BufWriter::new(File::create(terry_path)?);
        terry_file.write_all(terry_data.as_bytes())?;

        // Pre-calculate the associations section.
        let assoc_logical = self.logical_associations();
        let assoc_transform = self.trasnform_associations();

        // Now the layer.
        let mut layer_data = String::new();

        layer_data.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<layer version=\"41\">
    <entities>"
        );

        layer_data.push_str(&self.battlefield_building_list().to_layer(self)?);
        layer_data.push_str(&self.battlefield_building_list_far().to_layer(self)?);
        layer_data.push_str(&self.capture_location_set().to_layer(self)?);
        //layer_data.push_str(&self.ef_line_list().to_layer(self)?);
        //layer_data.push_str(&self.go_outlines().to_layer(self)?);
        //layer_data.push_str(&self.non_terrain_outlines().to_layer(self)?);
        //layer_data.push_str(&self.zones_template_list().to_layer(self)?);
        layer_data.push_str(&self.prefab_instance_list().to_layer(self)?);
        //layer_data.push_str(&self.bmd_outline_list().to_layer(self)?);
        //layer_data.push_str(&self.terrain_outlines().to_layer(self)?);
        //layer_data.push_str(&self.lite_building_outlines().to_layer(self)?);
        //layer_data.push_str(&self.camera_zones().to_layer(self)?);
        //layer_data.push_str(&self.civilian_deployment_list().to_layer(self)?);
        //layer_data.push_str(&self.civilian_shelter_list().to_layer(self)?);
        //layer_data.push_str(&self.prop_list().to_layer(self)?);
        //layer_data.push_str(&self.particle_emitter_list().to_layer(self)?);
        //layer_data.push_str(&self.ai_hints().to_layer(self)?);
        //layer_data.push_str(&self.light_probe_list().to_layer(self)?);
        //layer_data.push_str(&self.terrain_stencil_triangle_list().to_layer(self)?);
        //layer_data.push_str(&self.point_light_list().to_layer(self)?);
        //layer_data.push_str(&self.building_projectile_emitter_list().to_layer(self)?);
        //layer_data.push_str(&self.playable_area().to_layer(self)?);
        //layer_data.push_str(&self.custom_material_mesh_list().to_layer(self)?);
        //layer_data.push_str(&self.terrain_stencil_blend_triangle_list().to_layer(self)?);
        //layer_data.push_str(&self.spot_light_list().to_layer(self)?);
        //layer_data.push_str(&self.sound_shape_list().to_layer(self)?);
        //layer_data.push_str(&self.composite_scene_list().to_layer(self)?);
        //layer_data.push_str(&self.deployment_list().to_layer(self)?);
        //layer_data.push_str(&self.bmd_catchment_area_list().to_layer(self)?);
        //layer_data.push_str(&self.toggleable_buildings_slot_list().to_layer(self)?);
        //layer_data.push_str(&self.terrain_decal_list().to_layer(self)?);
        //layer_data.push_str(&self.tree_list_reference_list().to_layer(self)?);
        //layer_data.push_str(&self.grass_list_reference_list().to_layer(self)?);
        //layer_data.push_str(&self.water_outlines().to_layer(self)?);

        // Vegetation items are entities in the layer too.
        if let Some(vegetation) = vegetation {
            layer_data.push_str(&vegetation.to_layer(self)?);
        }

        layer_data.push_str("
    </entities>
    <associations>");

        if assoc_logical.is_empty() {
            layer_data.push_str("
        <Logical/>");
        } else {
            layer_data.push_str("
        <Logical>");

            for (key, values) in &assoc_logical {
                layer_data.push_str(&format!("
            <from id=\"{}\">",
                    key
                ));

                for value in values {
                    layer_data.push_str(&format!("
                <to id=\"{}\"/>",
                        value
                    ));
                }

                layer_data.push_str("
            </from>");
            }

            layer_data.push_str("
        </Logical>");
        }

        if assoc_transform.is_empty() {
            layer_data.push_str("
        <Transform/>");
        } else {

            layer_data.push_str("
        <Transform>");

            for (key, values) in &assoc_transform {
                layer_data.push_str(&format!("
            <from id=\"{}\">",
                    key
                ));

                for value in values {
                    layer_data.push_str(&format!("
                <to id=\"{}\"/>",
                        value
                    ));
                }

                layer_data.push_str("
            </from>");
            }

            layer_data.push_str("
        </Transform>");
        }

        layer_data.push_str("
    </associations>
</layer>
        ");

        let mut layer_file = BufWriter::new(File::create(layer_path)?);
        layer_file.write_all(layer_data.as_bytes())?;

        Ok(())
    }

    pub fn logical_associations(&self) -> HashMap<String, Vec<String>> {
        HashMap::new()
    }

    pub fn trasnform_associations(&self) -> HashMap<String, Vec<String>> {
        HashMap::new()
    }
}
