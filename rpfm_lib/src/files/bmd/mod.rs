//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Battle Map Definition (BMD) file format support.
//!
//! BMD files (`.bmd`) are FASTBIN0-format files that define battle map layouts for Total War
//! games. They contain comprehensive 3D scene data including buildings, terrain, lighting,
//! vegetation, deployment zones, AI hints, and other gameplay-critical map elements.
//!
//! # File Format
//!
//! BMD files use the FASTBIN0 binary format with a version-specific structure:
//!
//! ```text
//! [8 bytes]  FASTBIN0 signature
//! [u16]      serialise_version
//! [...]      version-specific data
//! ```
//!
//! # Supported Versions
//!
//! All currently supported versions are:
//! - **Version 23**
//! - **Version 24**
//! - **Version 25**
//! - **Version 26**
//! - **Version 27**
//!
//! # File Contents
//!
//! BMD files contain numerous specialized data lists:
//!
//! ## Buildings & Structures
//! - `BattlefieldBuildingList` - Buildings inside the battlefield area
//! - `BattlefieldBuildingListFar` - Buildings outside the battlefield area
//! - `PrefabInstanceList` - Prefab object instances
//! - `PropList` - Small decorative props
//!
//! ## Terrain & Outlines
//! - `TerrainOutlines` - Terrain boundary definitions
//! - `NonTerrainOutlines` - Non-terrain area boundaries
//! - `GoOutlines` - Traversable areas (where units can go)
//! - `WaterOutlines` - Water body boundaries
//!
//! ## Gameplay Elements
//! - `DeploymentList` - Unit deployment zones
//! - `CaptureLocationSet` - Groups of capture locations
//! - `PlayableArea` - Playable map boundaries
//! - `AIHints` - AI pathfinding and behavior hints
//!
//! ## Lighting & Visual Effects
//! - `PointLightList` - Point light sources
//! - `SpotLightList` - Spotlight sources
//! - `LightProbeList` - Global illumination probes
//! - `ParticleEmitterList` - Particle effect emitters
//!
//! ## Vegetation
//! - `TreeListReferenceList` - Tree placement references
//! - `GrassListReferenceList` - Grass placement references
//!
//! ## Other
//! - `CameraZones` - Camera constraint zones
//! - `SoundShapeList` - 3D audio zones
//! - `CompositeSceneList` - Composite scene references
//! - `CustomMaterialMeshList` - Custom material meshes
//! - And 20+ more specialized lists
//!
//! # Usage
//!
//! ```ignore
//! use rpfm_lib::files::bmd::Bmd;
//! use rpfm_lib::files::Decodeable;
//!
//! // Decode a BMD file
//! let bmd = Bmd::decode(&mut reader, &None)?;
//!
//! println!("BMD version: {}", bmd.serialise_version());
//!
//! // Access building list
//! for building in bmd.battlefield_building_list().list() {
//!     println!("Building: {:?}", building.key());
//! }
//!
//! // Export to Terry (CA's editor format)
//! bmd.export_prefab_to_raw_data("map_name", Some(&vegetation), &output_path)?;
//! ```
//!
//! # Terry Export
//!
//! BMD files can be exported to Terry (Creative Assembly's map editor) format using
//! [`Bmd::export_prefab_to_raw_data()`]. This generates:
//! - `.terry` project file
//! - `.layer` scene layer file with entities and associations
//!
//! # File Location
//!
//! BMD files are typically found in:
//! ```text
//! terrain/battles/*.bmd
//! terrain/tiles/*.bmd
//! ```

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

/// File extension for Battle Map Definition files.
///
/// BMD files use the `.bmd` extension.
pub const EXTENSIONS: [&str; 1] = [
    ".bmd",
];

/// FASTBIN0 file signature.
///
/// All BMD files start with this 8-byte signature: `FASTBIN0`
/// (bytes: `[0x46, 0x41, 0x53, 0x54, 0x42, 0x49, 0x4E, 0x30]`)
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

/// Represents a complete Battle Map Definition file decoded in memory.
///
/// This struct contains all battle map data including buildings, terrain, lighting,
/// deployment zones, AI hints, and numerous other gameplay and visual elements.
///
/// # Field Categories
///
/// ## Core
/// - `serialise_version`: File format version (23-27)
///
/// ## Buildings & Objects
/// - `battlefield_building_list` - Buildings inside the battlefield area
/// - `battlefield_building_list_far` - Buildings outside the battlefield area
/// - `prefab_instance_list`: Prefab object instances
/// - `prop_list`: Small decorative props
/// - `composite_scene_list`: Composite scene references
///
/// ## Terrain & Boundaries
/// - `terrain_outlines`: Terrain area boundaries
/// - `non_terrain_outlines`: Non-terrain area boundaries
/// - `go_outlines`: Traversable area outlines (where units can go).
/// - `water_outlines`: Water body boundaries
/// - `bmd_outline_list`: Additional outline definitions
/// - `lite_building_outlines`: Simplified building outlines
/// - `playable_area`: Playable map boundaries
///
/// ## Gameplay
/// - `deployment_list`: Unit deployment zones
/// - `capture_location_set`: Capture point locations
/// - `bmd_catchment_area_list`: Catchment area definitions
/// - `zones_template_list`: Zone template definitions
/// - `ef_line_list`: Entity Formation line definitions
/// - `ai_hints`: AI pathfinding and behavior hints
///
/// ## Lighting
/// - `point_light_list`: Point light sources
/// - `spot_light_list`: Spotlight sources
/// - `light_probe_list`: Global illumination probes
///
/// ## Effects & Audio
/// - `particle_emitter_list`: Particle effect emitters
/// - `sound_shape_list`: 3D audio zones
/// - `building_projectile_emitter_list`: Building-based projectile emitters
///
/// ## Vegetation
/// - `tree_list_reference_list`: Tree placement references
/// - `grass_list_reference_list`: Grass placement references
///
/// ## Advanced Rendering
/// - `custom_material_mesh_list`: Custom material meshes
/// - `terrain_stencil_triangle_list`: Terrain stencil geometry
/// - `terrain_stencil_blend_triangle_list`: Blended stencil geometry
/// - `terrain_decal_list`: Terrain decal placements
///
/// ## Civilians & Sieges
/// - `civilian_deployment_list`: Civilian unit spawns
/// - `civilian_shelter_list`: Civilian shelter locations
/// - `toggleable_buildings_slot_list`: Destructible building slots
///
/// ## Cameras
/// - `camera_zones`: Camera constraint volumes
///
/// # Getset
///
/// All fields have public getters, mutable getters, and setters generated via `getset`:
/// ```ignore
/// let version = bmd.serialise_version();  // Getter
/// *bmd.serialise_version_mut() = 27;      // Mutable getter
/// bmd.set_serialise_version(27);          // Setter
/// ```
///
/// # Example
///
/// ```ignore
/// use rpfm_lib::files::bmd::Bmd;
/// use rpfm_lib::files::Decodeable;
///
/// let bmd = Bmd::decode(&mut reader, &None)?;
///
/// // Check version
/// println!("BMD version: {}", bmd.serialise_version());
///
/// // Iterate buildings
/// for building in bmd.battlefield_building_list().list() {
///     println!("Building key: {:?}", building.key());
/// }
///
/// // Access deployment zones
/// for zone in bmd.deployment_list().list() {
///     println!("Deployment zone: {:?}", zone);
/// }
/// ```
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Bmd {
    /// File format version number (23-27).
    serialise_version: u16,

    /// Building instances inside the battlefield area.
    battlefield_building_list: BattlefieldBuildingList,

    /// Building instances outside the battlefield area.
    battlefield_building_list_far: BattlefieldBuildingListFar,

    /// Groups of capture locations.
    capture_location_set: CaptureLocationSet,

    /// Entity Formation line definitions.
    ef_line_list: EFLineList,

    /// Traversable area outlines (where units can go).
    go_outlines: GoOutlines,

    /// Non-terrain area boundary outlines.
    non_terrain_outlines: NonTerrainOutlines,

    /// Zone template definitions.
    zones_template_list: ZonesTemplateList,

    /// Prefab object instances.
    prefab_instance_list: PrefabInstanceList,

    /// Additional outline definitions.
    bmd_outline_list: BmdOutlineList,

    /// Terrain area boundary outlines.
    terrain_outlines: TerrainOutlines,

    /// Simplified building outline definitions.
    lite_building_outlines: LiteBuildingOutlines,

    /// Camera constraint volumes.
    camera_zones: CameraZones,

    /// Civilian unit deployment locations.
    civilian_deployment_list: CivilianDeploymentList,

    /// Civilian shelter locations.
    civilian_shelter_list: CivilianShelterList,

    /// Small decorative prop instances.
    prop_list: PropList,

    /// Particle effect emitter instances.
    particle_emitter_list: ParticleEmitterList,

    /// AI pathfinding and behavior hints.
    ai_hints: AIHints,

    /// Global illumination light probes.
    light_probe_list: LightProbeList,

    /// Terrain stencil triangle geometry.
    terrain_stencil_triangle_list: TerrainStencilTriangleList,

    /// Point light source instances.
    point_light_list: PointLightList,

    /// Building-based projectile emitter instances.
    building_projectile_emitter_list: BuildingProjectileEmitterList,

    /// Playable map area boundaries.
    playable_area: PlayableArea,

    /// Custom material mesh instances.
    custom_material_mesh_list: CustomMaterialMeshList,

    /// Blended terrain stencil triangle geometry.
    terrain_stencil_blend_triangle_list: TerrainStencilBlendTriangleList,

    /// Spotlight source instances.
    spot_light_list: SpotLightList,

    /// 3D audio zone definitions.
    sound_shape_list: SoundShapeList,

    /// Composite scene references.
    composite_scene_list: CompositeSceneList,

    /// Unit deployment zone definitions.
    deployment_list: DeploymentList,

    /// Catchment area definitions.
    bmd_catchment_area_list: BmdCatchmentAreaList,

    /// Destructible/toggleable building slot definitions.
    toggleable_buildings_slot_list: ToggleableBuildingsSlotList,

    /// Terrain decal placements.
    terrain_decal_list: TerrainDecalList,

    /// Tree placement references (links to vegetation files).
    tree_list_reference_list: TreeListReferenceList,

    /// Grass placement references (links to vegetation files).
    grass_list_reference_list: GrassListReferenceList,

    /// Water body boundary outlines.
    water_outlines: WaterOutlines,
}

//---------------------------------------------------------------------------//
//                           Implementation of Bmd
//---------------------------------------------------------------------------//

/// Trait for converting BMD data structures to Terry `.layer` XML format.
///
/// This trait is implemented by all BMD data list types to enable export to
/// Creative Assembly's Terry map editor format. Each implementation converts
/// its data to XML entity definitions that can be imported into Terry.
pub trait ToLayer {
    /// Converts this data structure to Terry `.layer` XML entity definitions.
    ///
    /// # Parameters
    ///
    /// - `parent`: Reference to the parent [`Bmd`] for accessing cross-referenced data
    ///
    /// # Returns
    ///
    /// - `Ok(String)`: XML entity definitions for this data list
    /// - `Err(_)`: Conversion error
    ///
    /// # Default Implementation
    ///
    /// Returns an empty string (no entities exported).
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
    /// Exports this BMD to Terry (Creative Assembly's editor) format.
    ///
    /// Generates two files for use in Terry:
    /// - `.terry` - Project file defining the prefab structure
    /// - `.layer` - Scene layer with entities, associations, and vegetation
    ///
    /// # Parameters
    ///
    /// - `name`: Base name for output files (e.g., "siege_map")
    /// - `vegetation`: Optional vegetation data from BMD vegetation file
    /// - `output_path`: Directory where files will be created
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Successfully exported files
    /// - `Err(_)`: I/O error or conversion error
    ///
    /// # Generated Files
    ///
    /// - `{name}.terry` - Project file with scene hierarchy
    /// - `{name}.187abf10b8b9a13.layer` - Layer file with entities
    ///
    /// # Entity Associations
    ///
    /// The layer file includes two association types:
    /// - **Logical**: Parent-child grouping relationships in Terry's UI
    /// - **Transform**: Spatial/hierarchical relationships
    ///
    /// # Example
    ///
    /// ```ignore
    /// use rpfm_lib::files::bmd::Bmd;
    /// use rpfm_lib::files::bmd_vegetation::BmdVegetation;
    /// use std::path::Path;
    ///
    /// let bmd = Bmd::decode(&mut reader, &None)?;
    /// let vegetation = BmdVegetation::decode(&mut veg_reader, &None)?;
    ///
    /// bmd.export_prefab_to_raw_data(
    ///     "my_battle_map",
    ///     Some(&vegetation),
    ///     Path::new("output/")
    /// )?;
    /// ```
    ///
    /// # Note
    ///
    /// Not all BMD data lists are currently exported. Some are commented out
    /// in the implementation and will be added as export support is completed.
    pub fn export_prefab_to_raw_data(&self, name: &str, vegetation: Option<&BmdVegetation>, output_path: &Path) -> Result<()> {

        // We need to generate two files:
        // - .terry: The project file with just one layer.
        // - .layer: The layer file with the contents of the bmd and bmd_vegetation.
        let terry_path = output_path.join(format!("{name}.terry"));
        let layer_path = output_path.join(format!("{name}.187abf10b8b9a13.layer"));

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
            <from id=\"{key}\">"
                ));

                for value in values {
                    layer_data.push_str(&format!("
                <to id=\"{value}\"/>"
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
            <from id=\"{key}\">"
                ));

                for value in values {
                    layer_data.push_str(&format!("
                <to id=\"{value}\"/>"
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

    /// Returns logical entity associations for Terry export.
    ///
    /// Logical associations define parent-child grouping relationships for
    /// organizing entities in Terry's UI hierarchy.
    ///
    /// # Returns
    ///
    /// Map of parent entity IDs to their logically grouped child entity IDs.
    ///
    /// # Note
    ///
    /// Currently returns an empty map. Will be populated as association
    /// logic is implemented.
    pub fn logical_associations(&self) -> HashMap<String, Vec<String>> {
        HashMap::new()
    }

    /// Returns transform (spatial hierarchy) entity associations for Terry export.
    ///
    /// Transform associations define parent-child spatial relationships between
    /// entities (e.g., props attached to buildings, lights attached to structures).
    ///
    /// # Returns
    ///
    /// Map of parent entity IDs to their child entity IDs.
    ///
    /// # Note
    ///
    /// Currently returns an empty map. Will be populated as association
    /// logic is implemented.
    pub fn trasnform_associations(&self) -> HashMap<String, Vec<String>> {
        HashMap::new()
    }
}
