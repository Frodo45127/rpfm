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

    pub(crate) fn read_v27<R: ReadBytes>(&mut self, data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.battlefield_building_list = BattlefieldBuildingList::decode(data, extra_data)?;
        self.battlefield_building_list_far = BattlefieldBuildingListFar::decode(data, extra_data)?;
        self.capture_location_set = CaptureLocationSet::decode(data, extra_data)?;
        self.ef_line_list = EFLineList::decode(data, extra_data)?;
        self.go_outlines = GoOutlines::decode(data, extra_data)?;
        self.non_terrain_outlines = NonTerrainOutlines::decode(data, extra_data)?;
        self.zones_template_list = ZonesTemplateList::decode(data, extra_data)?;
        self.prefab_instance_list = PrefabInstanceList::decode(data, extra_data)?;
        self.bmd_outline_list = BmdOutlineList::decode(data, extra_data)?;
        self.terrain_outlines = TerrainOutlines::decode(data, extra_data)?;
        self.lite_building_outlines = LiteBuildingOutlines::decode(data, extra_data)?;
        self.camera_zones = CameraZones::decode(data, extra_data)?;
        self.civilian_deployment_list = CivilianDeploymentList::decode(data, extra_data)?;
        self.civilian_shelter_list = CivilianShelterList::decode(data, extra_data)?;
        self.prop_list = PropList::decode(data, extra_data)?;
        self.particle_emitter_list = ParticleEmitterList::decode(data, extra_data)?;
        self.ai_hints = AIHints::decode(data, extra_data)?;
        self.light_probe_list = LightProbeList::decode(data, extra_data)?;
        self.terrain_stencil_triangle_list = TerrainStencilTriangleList::decode(data, extra_data)?;
        self.point_light_list = PointLightList::decode(data, extra_data)?;
        self.building_projectile_emitter_list = BuildingProjectileEmitterList::decode(data, extra_data)?;
        self.playable_area = PlayableArea::decode(data, extra_data)?;
        self.custom_material_mesh_list = CustomMaterialMeshList::decode(data, extra_data)?;
        self.terrain_stencil_blend_triangle_list = TerrainStencilBlendTriangleList::decode(data, extra_data)?;
        self.spot_light_list = SpotLightList::decode(data, extra_data)?;
        self.sound_shape_list = SoundShapeList::decode(data, extra_data)?;
        self.composite_scene_list = CompositeSceneList::decode(data, extra_data)?;
        self.deployment_list = DeploymentList::decode(data, extra_data)?;
        self.bmd_catchment_area_list = BmdCatchmentAreaList::decode(data, extra_data)?;
        self.toggleable_buildings_slot_list = ToggleableBuildingsSlotList::decode(data, extra_data)?;
        self.terrain_decal_list = TerrainDecalList::decode(data, extra_data)?;
        self.tree_list_reference_list = TreeListReferenceList::decode(data, extra_data)?;
        self.grass_list_reference_list = GrassListReferenceList::decode(data, extra_data)?;
        self.water_outlines = WaterOutlines::decode(data, extra_data)?;

        Ok(())
    }

    pub(crate) fn write_v27<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
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
