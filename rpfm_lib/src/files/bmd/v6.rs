//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
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

/// Versions 4 to 6, used by Attila. Version 5 adds the civilian deployment list, and version 6 the civilian shelter list.
///
/// Props and particle emitters are markers, and the AI hint lists have no version and are always empty.
impl Bmd {

    pub(crate) fn read_v4_to_v6<R: ReadBytes>(&mut self, data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.prop_list.read_markers(data, extra_data)?;
        self.particle_emitter_list.read_markers(data, extra_data)?;

        for section in ["AI hint polylines", "AI hint polyline lists", "AI separators"] {
            if data.read_u32()? != 0 {
                return Err(RLibError::DecodingFastBinUnknownSection(section.to_owned()));
            }
        }

        self.deployment_list = DeploymentList::decode(data, extra_data)?;
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

        if self.serialise_version >= 5 {
            self.civilian_deployment_list = CivilianDeploymentList::decode(data, extra_data)?;
        }

        if self.serialise_version >= 6 {
            self.civilian_shelter_list = CivilianShelterList::decode(data, extra_data)?;
        }

        Ok(())
    }

    pub(crate) fn write_v4_to_v6<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        self.prop_list.write_markers(buffer, extra_data)?;
        self.particle_emitter_list.write_markers(buffer, extra_data)?;

        // The AI hint lists, always empty.
        buffer.write_u32(0)?;
        buffer.write_u32(0)?;
        buffer.write_u32(0)?;

        self.deployment_list.encode(buffer, extra_data)?;
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

        if self.serialise_version >= 5 {
            self.civilian_deployment_list.encode(buffer, extra_data)?;
        }

        if self.serialise_version >= 6 {
            self.civilian_shelter_list.encode(buffer, extra_data)?;
        }

        Ok(())
    }
}
