//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This is a module to read/write Battle Map Definition binary (FASTBIN0) files.

use nalgebra::{Matrix3, Vector3, Rotation3, Matrix4};

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{Decodeable, EncodeableExtraData, Encodeable};
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
use super::DecodeableExtraData;

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

mod common;
mod v23;
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

// TODO: Move properties, property_overrides and flags to common.


// No overloading supported sadly :(
// and idk how to use generics with structs


/*
    Restores normal Rotation matrix representation as on the picture:
    https://developer.unigine.com/forum/uploads/monthly_2020_05/image.png.674c8b961433f2a7a62c54bc55cb599c.png
    pic note: (it seems like R should be applies to each column)
    from CA's column-first serialization

*/
fn create_rotation_matrix_from_transform3x4(t: &Transform3x4) -> Matrix3<f64>{
    // Fix order of the elements here
    let matrix = Matrix3::new(
        *t.m00() as f64, *t.m10() as f64, *t.m20() as f64,
        *t.m01() as f64, *t.m11() as f64, *t.m21() as f64,
        *t.m02() as f64, *t.m12() as f64, *t.m22() as f64
    );
    matrix
}

fn create_rotation_matrix_from_transform4x4(t: &Transform4x4) -> Matrix3<f64>{
    // Fix order of the elements here
    let matrix = Matrix3::new(
        *t.m00() as f64, *t.m10() as f64, *t.m20() as f64,
        *t.m01() as f64, *t.m11() as f64, *t.m21() as f64,
        *t.m02() as f64, *t.m12() as f64, *t.m22() as f64
    );
    matrix
}

// fn math_transforms_to_transform3x4(m: &Matrix3<f64>, translate: (f64, f64, f64)) -> Transform3x4{
//     // Fix order of the elements here
//     Transform3x4 {
//         // rotation matrix in column-first order
//         m00: m[(0,0)] as f32,
//         m01: m[(1,0)] as f32,
//         m02: m[(2,0)] as f32,
//         m10: m[(0,1)] as f32,
//         m11: m[(1,1)] as f32,
//         m12: m[(2,1)] as f32,
//         m20: m[(0,2)] as f32,
//         m21: m[(1,2)] as f32,
//         m22: m[(2,2)] as f32,
//         //translation vector
//         m30: translate.0 as f32,
//         m31: translate.1 as f32,
//         m32: translate.2 as f32
//     };
// }

// fn math_transforms_to_transform4x4(m: &Matrix3<f64>, translate: (f64, f64, f64)) -> Transform4x4{
//     // Fix order of the elements here
//     Transform4x4 {
//         // rotation matrix in column-first order
//         m00: m[(0,0)] as f32,
//         m01: m[(1,0)] as f32,
//         m02: m[(2,0)] as f32,
//         m10: m[(0,1)] as f32,
//         m11: m[(1,1)] as f32,
//         m12: m[(2,1)] as f32,
//         m20: m[(0,2)] as f32,
//         m21: m[(1,2)] as f32,
//         m22: m[(2,2)] as f32,
//         //translation vector
//         m30: translate.0 as f32,
//         m31: translate.1 as f32,
//         m32: translate.2 as f32,
//         //just fill
//         m03: 0 as f32,
//         m13: 0 as f32,
//         m23: 0 as f32,
//         m33: 1 as f32
//     };
// }


/*
Extracts scales as described here:
https://math.stackexchange.com/a/1463487
DOES NOT SUPPORT NEGATIVE SCALES
 */
fn extract_scales(matrix: Matrix3<f64>) -> (f64, f64, f64) {
    let scale = (
        matrix.column(0).norm(),
        matrix.column(1).norm(),
        matrix.column(2).norm()
    );
    scale
}

fn apply_scales(matrix: Matrix3<f64>, scales: (f64, f64, f64)) -> Matrix3<f64> {
    let scaled_matrix = Matrix3::new(
        matrix[(0, 0)] * scales.0, matrix[(0, 1)] * scales.1, matrix[(0, 2)] * scales.2,
        matrix[(1, 0)] * scales.0, matrix[(1, 1)] * scales.1, matrix[(1, 2)] * scales.2,
        matrix[(2, 0)] * scales.0, matrix[(2, 1)] * scales.1, matrix[(2, 2)] * scales.2,
    );
    scaled_matrix
}

fn normalize_rotation_matrix(matrix: Matrix3<f64>, scales: (f64, f64, f64)) -> Matrix3<f64> {
    let normalized_matrix = Matrix3::new(
        matrix[(0, 0)] / scales.0, matrix[(0, 1)] / scales.1, matrix[(0, 2)] / scales.2,
        matrix[(1, 0)] / scales.0, matrix[(1, 1)] / scales.1, matrix[(1, 2)] / scales.2,
        matrix[(2, 0)] / scales.0, matrix[(2, 1)] / scales.1, matrix[(2, 2)] / scales.2,
    );
    normalized_matrix
}

/*
As I understand it uses 'xyz' extrinsic rotations order
Python code using scipy lib:
    r = Rotation.from_euler("xyz", [-130.00000555832042, 80.00000457701574, -29.999991697018082], degrees=True)
    m = r.as_matrix()
    r = Rotation.from_matrix(m)
    angles = r.as_euler("xyz", degrees=True)
 */
fn rotation_matrix_to_euler_angles(matrix: Matrix3<f64>, degrees: bool) -> (f64, f64, f64) {
    let rotation = Rotation3::from_matrix_unchecked(matrix);
    let euler = rotation.euler_angles();
    let euler_angles = if degrees {
        (
            euler.0.to_degrees(),
            euler.1.to_degrees(),
            euler.2.to_degrees(),
        )
    } else {
       (euler.0, euler.1, euler.2)
    };
    euler_angles
}

fn euler_angles_to_rotation_matrix(angles: (f64, f64, f64), degrees: bool) -> Matrix3<f64> {
    let _angles = if degrees {
        (
            angles.0.to_radians(),
            angles.1.to_radians(),
            angles.2.to_radians(),
        )
    } else {
        angles
    };
    let rotation = Rotation3::from_euler_angles(_angles.0, _angles.1, _angles.2);
    let mut matrix : Matrix3<f64> = rotation.into();

    //prettify
    matrix.iter_mut().for_each(|element| {
        if element.abs() < 1e-5 {
            *element = 0.0;
        }
    });
    matrix

}

impl Bmd {

    pub fn to_layer(&self) -> Result<String> {

        let mut layer = String::new();
        layer.push_str("
        <?xml version=\"1.0\" encoding=\"UTF-8\"?>
        <layer version=\"41\">
            <entities>
        ");


        for prefab in self.prefab_instance_list.prefab_instances() {
            let rotation_matrix = create_rotation_matrix_from_transform4x4(prefab.transform());
            let scales = extract_scales(rotation_matrix);
            let normalized_rotation_matrix = normalize_rotation_matrix(rotation_matrix, scales);
            let angles= rotation_matrix_to_euler_angles(normalized_rotation_matrix, true);
            let position = Vector3::new(
                (*prefab.transform().m30()) as f64,
                (*prefab.transform().m31()) as f64,
                (*prefab.transform().m32()) as f64,

            );

            //debug
            let dmatrix = apply_scales(
                euler_angles_to_rotation_matrix(angles, true),
                scales
            );


            println!("Prefab Transform:\n{:?}", prefab.transform());
            println!("Position:\n{:?}", position);
            println!("Orig Matrix:\n{:?}", rotation_matrix);
            println!("Angles:\n{:?}", angles);
            println!("Result Matrix:\n{:?}", dmatrix);
            println!("\n");
        }



        // Battlefield Buildings
        for building in self.battlefield_building_list.buildings() { //.battlefield_building_list.buildings()
            layer.push_str(&format!("<entity id=\"{:x}\">", building.uid()));

            layer.push_str(&format!("<ECBuilding
                key=\"{}\"
                damage=\"{}\"
                indestructible=\"{}\"
                toggleable=\"{}\"
                key_building=\"{}\"
                hide_tooltip=\"{}\"
                settlement_level_configurable=\"{}\"
                capture_location=\"\"
                export_as_prop=\"false\"
                visible_beyond_outfield=\"false\"/>",
                building.building_key(),
                building.properties().starting_damage_unary(),
                building.properties().indestructible(),
                building.properties().toggleable(),
                building.properties().key_building(),
                building.properties().hide_tooltip(),
                building.properties().settlement_level_configurable(),
            ));

            layer.push_str(&format!("<ECMeshRenderSettings
                cast_shadow=\"{}\"
                receive_decals=\"true\"
                render_into_skydome_fog=\"false\"/>",
                building.properties().cast_shadows()
            ));

            layer.push_str(&format!("<ECVisibilitySettingsBattle visible_in_tactical_view=\"true\" visible_in_tactical_view_only=\"false\"/>"));

            layer.push_str(&format!("<ECWater is_water=\"false\"/>"));

            let rotation_matrix = create_rotation_matrix_from_transform3x4(building.transform());
            let scales = extract_scales(rotation_matrix);
            let normalized_rotation_matrix = normalize_rotation_matrix(rotation_matrix, scales);
            let angles= rotation_matrix_to_euler_angles(normalized_rotation_matrix, true);

            layer.push_str(&format!("<ECTransform
                position=\"{:.5} {:.5} {:.5}\"
                rotation=\"{:.5} {:.5} {:.5}\"
                scale=\"{:.5} {:.5} {:.5}\"
                pivot=\"0 0 0\"/>",
                building.transform().m30(), building.transform().m31(), building.transform().m32(),
                angles.0, angles.1, angles.2,
                scales.0, scales.1, scales.2
            ));

            //debug
            let dmatrix = apply_scales(
                euler_angles_to_rotation_matrix(angles, true),
                scales
            );

            println!("Orig Matrix:\n{:?}", rotation_matrix);
            println!("Angles:\n{:?}", angles);
            println!("Result Matrix:\n{:?}", dmatrix);
            println!("\n");

            // Ok, I'm shit at math and haven't touched matrixes in 12 years....
            // Position:
            //  - x: m30()
            //  - y: m31(). Note that if it's clamped to terrain (height_mode == "BHM_TERRAIN" or "BHM_TERRAIN_ALIGN_ORIENTATION") it's 0.
            //  - z: m32()
            //
            // Scale:
            //  - x: (m00() * m00()) + (m01() * m01()) + (m02() * m02())
            //  - y: (m10() * m10()) + (m11() * m11()) + (m12() * m12())
            //  - z: (m20() * m20()) + (m21() * m21()) + (m22() * m22())
            //
            // Rotation:
            //  - x: ?
            //  - y: ?
            //  - z: ?
            //
            // Pivot:
            //  - x: ?
            //  - y: ?
            //  - z: ?
            //
      /*

        position=\"0.530761719 1.12056732e-05 -8.57043457\"
        rotation=\"0 109.999954 0\"
        scale=\"0.999999821 1 0.999999821\"
        pivot=\"0 0 0\"/>",

              <transform
            m00='-0.342019' m01='0.000000' m02='-0.939693'
            m10='0.000000' m11='1.000000' m12='0.000000'
            m20='0.939693' m21='0.000000' m22='-0.342019'
            m30='0.530762' m31='0.000000' m32='-8.570435'/>
    */
            layer.push_str(&format!("<ECTerrainClamp
                active=\"{}\"
                clamp_to_sea_level=\"false\"
                terrain_oriented=\"{}\"
                fit_height_to_terrain=\"false\"/>",
                *building.properties().clamp_to_surface() || building.height_mode() == "BHM_TERRAIN" || building.height_mode() == "BHM_TERRAIN_ALIGN_ORIENTATION",
                building.height_mode() == "BHM_TERRAIN_ALIGN_ORIENTATION"
            ));

            layer.push_str(&format!("<ECPrefabOverride enabled=\"false\" id=\"{}\"/>", building.building_key()));


    /*
      <ECBuilding key="wh_glb_bucket_01" damage="0" indestructible="false" toggleable="false" key_building="false" hide_tooltip="false" settlement_level_configurable="false" capture_location="" export_as_prop="false" visible_beyond_outfield="false"/>
      <ECMeshRenderSettings cast_shadow="true" receive_decals="true" render_into_skydome_fog="false"/>
      <ECVisibilitySettingsBattle visible_in_tactical_view="true" visible_in_tactical_view_only="false"/>
      <ECWater is_water="false"/>
      <ECTransform position="0.530761719 1.12056732e-05 -8.57043457" rotation="0 109.999954 0" scale="0.999999821 1 0.999999821" pivot="0 0 0"/>
      <ECTerrainClamp active="true" clamp_to_sea_level="false" terrain_oriented="false" fit_height_to_terrain="false"/>
      <ECPrefabOverride enabled="false" id="wh_glb_bucket_01"/>

        <BUILDING serialise_version='11' building_id='' parent_id='-1' building_key='wh_glb_bucket_01' position_type='BBPT_LF_RELATIVE' height_mode='BHM_TERRAIN' uid='110298702775184652'>
        <transform
            m00='-0.342019' m01='0.000000' m02='-0.939693'
            m10='0.000000' m11='1.000000' m12='0.000000'
            m20='0.939693' m21='0.000000' m22='-0.342019'
            m30='0.530762' m31='0.000000' m32='-8.570435'/>
        <properties serialise_version='11' building_id='' starting_damage_unary='0.000000' on_fire='false' start_disabled='false' weak_point='false' ai_breachable='true' indestructible='false' dockable='true' toggleable='false'
            lite='false' cast_shadows='true' key_building='false' key_building_use_fort='false' is_prop_in_outfield='false' settlement_level_configurable='false' hide_tooltip='false' include_in_fog='false'/>
    */
            layer.push_str("</entity>");
        }

        layer.push_str("
            </entities>
            <associations>
                <Logical/>
                <Transform/>
            </associations>
        </layer>
        ");

        Ok(layer)
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
            26 => self.write_v26(buffer, extra_data)?,
            27 => self.write_v27(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("Bmd"), self.serialise_version)),
        }

        Ok(())
    }
}
