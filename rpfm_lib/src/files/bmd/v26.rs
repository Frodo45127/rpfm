//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use crate::binary::ReadBytes;
use crate::error::Result;
use crate::files::Decodeable;

use super::*;

//---------------------------------------------------------------------------//
//                           Implementation of Bmd
//---------------------------------------------------------------------------//

impl Bmd {

    pub(crate) fn read_v26<R: ReadBytes>(&mut self, data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        dbg!(data.len()?);
        self.battlefield_building_list = BattlefieldBuildingList::decode(data, extra_data)?;
        dbg!(1);
        self.battlefield_building_list_far = BattlefieldBuildingListFar::decode(data, extra_data)?;
        dbg!(1);
        self.capture_location_set = CaptureLocationSet::decode(data, extra_data)?;
        dbg!(1);
        self.ef_line_list = EFLineList::decode(data, extra_data)?;
        dbg!(1);
        self.go_outlines = GoOutlines::decode(data, extra_data)?;
        dbg!(1);
        self.non_terrain_outlines = NonTerrainOutlines::decode(data, extra_data)?;
        dbg!(1);
        self.zones_template_list = ZonesTemplateList::decode(data, extra_data)?;
        dbg!(1);
        self.prefab_instance_list = PrefabInstanceList::decode(data, extra_data)?;
        dbg!(1);
        self.bmd_outline_list = BmdOutlineList::decode(data, extra_data)?;
        dbg!(1);
        self.terrain_outlines = TerrainOutlines::decode(data, extra_data)?;
        dbg!(1);
        self.lite_building_outlines = LiteBuildingOutlines::decode(data, extra_data)?;
        dbg!(1);
        self.camera_zones = CameraZones::decode(data, extra_data)?;
        dbg!(1);
        self.civilian_deployment_list = CivilianDeploymentList::decode(data, extra_data)?;
        dbg!(1);
        self.civilian_shelter_list = CivilianShelterList::decode(data, extra_data)?;
        dbg!(1);
        self.prop_list = PropList::decode(data, extra_data)?;
        dbg!(1);
        self.particle_emitter_list = ParticleEmitterList::decode(data, extra_data)?;
        dbg!(1);
        self.ai_hints = AIHints::decode(data, extra_data)?;
        dbg!(1);
        self.light_probe_list = LightProbeList::decode(data, extra_data)?;
        dbg!(1);
        self.terrain_stencil_triangle_list = TerrainStencilTriangleList::decode(data, extra_data)?;
        dbg!(1);
        self.point_light_list = PointLightList::decode(data, extra_data)?;
        dbg!(1);
        self.building_projectile_emitter_list = BuildingProjectileEmitterList::decode(data, extra_data)?;
        dbg!(1);
        self.playable_area = PlayableArea::decode(data, extra_data)?;
        dbg!(1);
        self.custom_material_mesh_list = CustomMaterialMeshList::decode(data, extra_data)?;
        dbg!(1);
        self.terrain_stencil_blend_triangle_list = TerrainStencilBlendTriangleList::decode(data, extra_data)?;
        dbg!(1);
        self.spot_light_list = SpotLightList::decode(data, extra_data)?;
        dbg!(1);
        self.sound_shape_list = SoundShapeList::decode(data, extra_data)?;
        dbg!(1);
        self.composite_scene_list = CompositeSceneList::decode(data, extra_data)?;
        dbg!(1);
        self.deployment_list = DeploymentList::decode(data, extra_data)?;
        dbg!(1);
        self.bmd_catchment_area_list = BmdCatchmentAreaList::decode(data, extra_data)?;
        dbg!(1);
        self.toggleable_buildings_slot_list = ToggleableBuildingsSlotList::decode(data, extra_data)?;
        dbg!(1);
        self.terrain_decal_list = TerrainDecalList::decode(data, extra_data)?;
        dbg!(1);
        self.tree_list_reference_list = TreeListReferenceList::decode(data, extra_data)?;
        dbg!(1);
        self.grass_list_reference_list = GrassListReferenceList::decode(data, extra_data)?;
        dbg!(data.stream_position()?);
        self.water_outlines = WaterOutlines::decode(data, extra_data)?;
        dbg!(1);

        Ok(())
    }

    pub(crate) fn write_v26<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        self.battlefield_building_list.encode(buffer, extra_data)?;
        self.battlefield_building_list_far.encode(buffer, extra_data)?;
        self.capture_location_set.encode(buffer, extra_data)?;
        self.ef_line_list.encode(buffer, extra_data)?;
        self.go_outlines.encode(buffer, extra_data)?;
        self.non_terrain_outlines.encode(buffer, extra_data)?;
        self.zones_template_list.encode(buffer, extra_data)?;
        self.prefab_instance_list.encode(buffer, extra_data)?;
        self.bmd_outline_list.encode(buffer, extra_data)?;
        self.terrain_outlines.encode(buffer, extra_data)?;
        self.lite_building_outlines.encode(buffer, extra_data)?;
        self.camera_zones.encode(buffer, extra_data)?;
        self.civilian_deployment_list.encode(buffer, extra_data)?;
        self.civilian_shelter_list.encode(buffer, extra_data)?;
        self.prop_list.encode(buffer, extra_data)?;
        self.particle_emitter_list.encode(buffer, extra_data)?;
        self.ai_hints.encode(buffer, extra_data)?;
        self.light_probe_list.encode(buffer, extra_data)?;
        self.terrain_stencil_triangle_list.encode(buffer, extra_data)?;
        self.point_light_list.encode(buffer, extra_data)?;
        self.building_projectile_emitter_list.encode(buffer, extra_data)?;
        self.playable_area.encode(buffer, extra_data)?;
        self.custom_material_mesh_list.encode(buffer, extra_data)?;
        self.terrain_stencil_blend_triangle_list.encode(buffer, extra_data)?;
        self.spot_light_list.encode(buffer, extra_data)?;
        self.sound_shape_list.encode(buffer, extra_data)?;
        self.composite_scene_list.encode(buffer, extra_data)?;
        self.deployment_list.encode(buffer, extra_data)?;
        self.bmd_catchment_area_list.encode(buffer, extra_data)?;
        self.toggleable_buildings_slot_list.encode(buffer, extra_data)?;
        self.terrain_decal_list.encode(buffer, extra_data)?;
        self.tree_list_reference_list.encode(buffer, extra_data)?;
        self.grass_list_reference_list.encode(buffer, extra_data)?;
        self.water_outlines.encode(buffer, extra_data)?;

        Ok(())
    }
}
