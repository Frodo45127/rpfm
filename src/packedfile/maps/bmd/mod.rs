// In this file we define the bmd_data for decoding and encoding it.
// NOTE: The length of all strings is defined in 2 bytes before the actual string
// The m00-m32 stuff is the transformation matrix. A transformation matrix
// always has the same amount of floats (4x3), so we can probably do that a bit
// more efficiently than how I've done it currently.

/// TODO:
/// BMD_CATCHMENT_AREA can still be done.
/// Could also still check (river) splines.

use common::coding_helpers;

/// Struct BmdData: This stores the data of a decoded Localisation PackedFile in memory.
/// It stores the PackedFile divided in 2 parts:
/// - packed_file_header: header of the PackedFile, decoded.
/// - packed_file_data: data of the PackedFile, decoded.
#[derive(Clone)]
pub struct Bmd {
    pub packed_file_header: BmdHeader,
    pub packed_file_data: BmdData,
}

/// Struct BmdDataHeader: This stores the header of a decoded BMD_data file in memory.
/// It stores the BmdData's header in different parts:
/// - bmd_file_header_bmd_file_type: string "FASTBIN0" (8 bytes)
#[derive(Clone)]
pub struct BmdHeader {
    pub bmd_file_header_bmd_file_type: String,
    pub bmd_file_header_bmd_file_version: u16,
}

/// Struct BmdData: This stores the data of a decoded Bmd_data file in memory.
/// Each 'section' has a header with the serialise_version and an int indicating the number of entries
/// It stores the PackedFile's data in a Vec<LocDataEntry>.
#[derive(Clone, Debug)]
pub struct BmdData {
    pub battlefield_building_list: Vec<BattlefieldBuildingList>,
    pub battlefield_building_list_far: Vec<BattlefieldBuildingList>,
    pub capture_location_set:Vec<CaptureLocationSet>,
    pub ef_line_list:Vec<EfLineList>,
    pub go_outlines:Vec<GoOutlines>,
    pub non_terrain_outlines:Vec<NonTerrainOutlines>,
    pub zones_template_list:Vec<ZonesTemplateList>,
    pub prefab_instance_list:Vec<PrefabInstanceList>,
    pub bmd_outline_list:Vec<BmdOutlineList>,
    pub terrain_outlines:Vec<TerrainOutlines>,
    pub lite_building_outlines:Vec<LiteBuildingOutlines>,
    pub camera_zones:Vec<CameraZones>,
    pub civilian_deployment_list:Vec<CivilianDeploymentList>,
    pub civilian_shelter_list:Vec<CivilianShelterList>,
    pub prop_list:Vec<PropList>,
    pub particle_emitter_list:Vec<ParticleEmitterList>,
    pub ai_hints:Vec<AiHints>,
    pub light_probe_list:Vec<LightProbeList>,
    pub terrain_stencil_triangle_list:Vec<TerrainStencilTriangleList>,
    pub point_light_list:Vec<PointLightList>,
    pub building_projectile_emitter_list:Vec<BuildingProjectileEmitterList>,
    pub playable_area:Vec<PlayableArea>,
    pub custom_material_mesh_list:Vec<CustomMaterialMeshList>,
    pub terrain_stencil_blend_triangle_list:Vec<TerrainStencilBlendTriangleList>,
    pub spot_light_list:Vec<SpotLightList>,
    pub sound_shape_list:Vec<SoundShapeList>,
    pub composite_scene_list:Vec<CompositeSceneList>,
    pub deployment_list:Vec<DeploymentList>,
    pub bmd_catchment_area_list:Vec<BmdCatchmentAreaList>,
}


/// START OF BUILDINGS ########################################################
/// Struct BattlefieldBuildingList: This stores the data for the battlefield
/// buildings. It doesn't really have a header, it just has the serialise
/// version and then it goes straight to the number of entries
#[derive(Clone, Debug)]
pub struct BattlefieldBuildingList{
    pub version: u16,
    pub num_entries: u32,
    pub building_list: Vec<BattlefieldBuilding>,
}
/// Building struct.
/// TODO:
/// check the building_key, which is used in CaptureLocationLinks and
/// BuildingProjectileEmitter
/// in both of those cases I've listed it as u32, but here it is listed as u16.
#[derive(Clone, Debug)]
pub struct BattlefieldBuilding {
    pub version: u16,
    pub building_id: u16, //not sure of the type, was always 0000
    pub parent_id: u16, //check
    pub building_key: String,
    pub position_type: String,
    pub m00: f32,
    pub m01: f32,
    pub m02: f32,
    pub m10: f32,
    pub m11: f32,
    pub m12: f32,
    pub m20: f32,
    pub m21: f32,
    pub m22: f32,
    pub m30: f32,
    pub m31: f32,
    pub m32: f32,
    pub properties_serialise_version: u32,
    pub building_id_2: u16, //actually also called building_id in the bmd_data
    pub starting_damage_unary: f32,
    pub mystery_bytes: u16, //unknown, check
    pub on_fire: bool,
    pub start_disabled: bool,
    pub weak_point: bool,
    pub ai_breachable: bool,
    pub indestructible: bool,
    pub dockable: bool,
    pub toggleable: bool,
    pub lite: bool,
    pub clamp_to_surface: bool,
    pub cast_shadows: bool,
    pub height_mode: String,
}
/// END OF BUILDINGS ##########################################################


#[derive(Clone, Debug)]
pub struct BattlefieldBuildingListFar{
    pub version: u16,
    pub num_entries: u32,
    pub building_far_list: Vec<BattlefieldBuildingFar>,
}

/// TODO: Define this.
#[derive(Clone, Debug)]
pub struct BattlefieldBuildingFar{
    pub nothing: Vec<u8>
}

/// START OF CAPTURE_LOCATIONS ################################################
/// CaptureLocationSet's contain CaptureLocationList's, which contain
/// CaptureLocation's, which contain CaptureLocationPoints and
/// CaptureLocationBuildingLinks
#[derive(Clone, Debug)]
pub struct CaptureLocationSet{
    pub version: u16,
    pub num_entries: u32,
    pub capture_location_set: Vec<CaptureLocationList>,
}
/// CaptureLocationList does not have a serialise_version
#[derive(Clone, Debug)]
pub struct CaptureLocationList{
    pub version: u16,
    pub capture_location_set: Vec<CaptureLocation>,
}
#[derive(Clone, Debug)]
pub struct CaptureLocation{
    pub location_x: f32,
    pub location_y: f32,
    pub radius: f32,
    pub valid_for_min_num_players: u32,
    pub valid_for_max_num_players: u32,
    pub capture_point_type: String,
    pub number_location_points: u32,
    pub capture_location_points: Vec<CaptureLocationPoints>, //can be multiple. is this the right way?
    pub database_key: String,
    pub flag_facing_x: f32,
    pub flag_facing_y: f32,
    pub number_building_links: u32,
    pub building_links: Vec<CaptureLocationBuildingLink>,
}
#[derive(Clone, Debug)]
pub struct CaptureLocationPoints{
    pub location_point_x: f32,
    pub location_point_y: f32,
}
#[derive(Clone, Debug)]
pub struct CaptureLocationBuildingLink{
    pub version: u16,
    pub building_index: u32, //check with BattlefieldBuilding
    pub prefab_index: u32, //was always set to -1, but that might not always be the case, especially with CA maps
    pub prefab_building_key: u16, //might be weird that this would be u16 while building_index is u32
}
/// END OF CAPTURE_LOCATIONS ##################################################


/// EF_LINE_LIST does not have a serialise_version
#[derive(Clone, Debug)]
pub struct EfLineList{
    pub num_entries: u32,
    pub ef_line_list: Vec<EfLine>,
}

/// TODO: Define this.
#[derive(Clone, Debug)]
pub struct EfLine{
    pub nothing: Vec<u8>
}

/// START OF GO_OUTLINES ######################################################
/// GO_OUTLINES does not have a serialise_version
#[derive(Clone, Debug)]
pub struct GoOutlines{
    pub num_entries: u32,
    pub go_lines: Vec<GoOutline>,
}
#[derive(Clone, Debug)]
pub struct GoOutline{
    pub num_points: u32,
    pub go_outline_points: Vec<GoOutlinePoints>,//can be multiple, as indicated in num_points
}
#[derive(Clone, Debug)]
pub struct GoOutlinePoints{
    pub position_x: f32,
    pub position_y: f32,
}
/// END OF GO_OUTLINES ########################################################


/// START OF NON_TERRAIN_OUTLINES #############################################
/// NON_TERRAIN_OUTLINES does not have a serialise_version
#[derive(Clone, Debug)]
pub struct NonTerrainOutlines{
    pub num_entries: u32,
    pub non_terrain_outline: Vec<NonTerrainOutline>,
}
#[derive(Clone, Debug)]
pub struct NonTerrainOutline{
    pub num_points: u32,
    pub go_outline_points: Vec<NonTerrainOutlinePoints>,//can be multiple, as indicated in num_points
}
#[derive(Clone, Debug)]
pub struct NonTerrainOutlinePoints{
    pub position_x: f32,
    pub position_y: f32,
}
/// END OF NON_TERRAIN_OUTLINES ###############################################

#[derive(Clone, Debug)]
pub struct ZonesTemplateList{
    pub version: u16,
    pub num_entries: u32,
    pub zones_template_list: Vec<ZonesTemplate>,
}

/// TODO: Define this.
#[derive(Clone, Debug)]
pub struct ZonesTemplate{
    pub nothing: Vec<u8>
}

/// START OF PREFAB ###########################################################
///TODO:
/// PrefabInstance also needs campaign_type_mask and campaign_region_key properly defined
/// they are 10 bytes together, not sure what that would add up to. both were empty each time
/// I should re-check this whole part. is probably more similar to the prop stuff.
///
/// I should re-check that, because I had listed 3 entries too many for the floats.
/// so that means I took 12 bytes which were not supposed to be floats?
#[derive(Clone, Debug)]
pub struct PrefabInstanceList{
    pub version: u16,
    pub num_entries: u32,
    pub prefab_instance_list: Vec<PrefabInstance>,
}
#[derive(Clone, Debug)]
pub struct PrefabInstance{
    pub version: u16,
    pub prefab_key: String,
    pub m00: f32,
    pub m01: f32,
    pub m02: f32,
    pub m10: f32,
    pub m11: f32,
    pub m12: f32,
    pub m20: f32,
    pub m21: f32,
    pub m22: f32,
    pub m30: f32,
    pub m31: f32,
    pub m32: f32,
    //pub campaign_type_mask:, //unknown
    //pub campaign_region_key:, //unknown
    pub clamp_to_surface: bool,
    pub height_mode: String,
}
/// END OF PREFAB #############################################################


#[derive(Clone, Debug)]
pub struct BmdOutlineList{
    pub version: u16,
    pub num_entries: u32,
    pub bmd_outline_list: Vec<BmdOutline>,
}

/// TODO: Define this.
#[derive(Clone, Debug)]
pub struct BmdOutline{
    pub nothing: Vec<u8>
}

/// TERRAIN_OUTLINES does not have a serialise_version
#[derive(Clone, Debug)]
pub struct TerrainOutlines{
    pub num_entries: u32,
    pub terrain_outlines: Vec<TerrainOutline>,
}

/// TODO: Define this.
#[derive(Clone, Debug)]
pub struct TerrainOutline{
    pub nothing: Vec<u8>
}


/// LITE_BUILDING_OUTLINES does not have a serialise_version
#[derive(Clone, Debug)]
pub struct LiteBuildingOutlines{
    pub num_entries: u32,
    pub lite_building_outlines: Vec<LiteBuildingOutline>,
}

/// TODO: Define this.
#[derive(Clone, Debug)]
pub struct LiteBuildingOutline{
    pub nothing: Vec<u8>
}


#[derive(Clone, Debug)]
pub struct CameraZones{
    pub version: u16,
    pub num_entries: u32,
    pub camera_zones: Vec<CameraZone>,
}

/// TODO: Define this.
#[derive(Clone, Debug)]
pub struct CameraZone{
    pub nothing: Vec<u8>
}


/// CIVILIAN_DEPLOYMENT_LIST does not have a serialise_version
#[derive(Clone, Debug)]
pub struct CivilianDeploymentList{
    pub num_entries: u32,
    pub civilian_deployment_list: Vec<CivilianDeployment>,
}

/// TODO: Define this.
#[derive(Clone, Debug)]
pub struct CivilianDeployment{
    pub nothing: Vec<u8>
}

/// CIVILIAN_SHELTER_LIST does not have a serialise_version
#[derive(Clone, Debug)]
pub struct CivilianShelterList{
    pub num_entries: u32,
    pub civilian_shelter_list: Vec<CivilianShelter>,
}

/// TODO: Define this.
#[derive(Clone, Debug)]
pub struct CivilianShelter{
    pub nothing: Vec<u8>
}


/// START OF PROP #############################################################
/// TODO: Prop
#[derive(Clone, Debug)]
pub struct PropList{
    pub version: u16,
    pub num_unique_props: u32, //number of unique props
    pub prop_keys: Vec<PropKeys>,
    pub prop_list: Vec<Prop>,
}
///key_index, as used in Prop, is defined by the position of the rigid_model_key in the
/// list of rigid_model_keys. So we need to keep track of that somehow.
#[derive(Clone, Debug)]
pub struct PropKeys{
    pub rigid_model_key: String,
}
///TODO:
#[derive(Clone, Debug)]
pub struct Prop{
    pub num_entries: u32, //absolute number of props
    pub version: u16, //check whether it's correct that this is after the num_entries
    pub key_index: u32,
    pub m00: f32,
    pub m01: f32,
    pub m02: f32,
    pub m10: f32,
    pub m11: f32,
    pub m12: f32,
    pub m20: f32,
    pub m21: f32,
    pub m22: f32,
    pub m30: f32,
    pub m31: f32,
    pub m32: f32,
    pub decal: bool,
    pub logic_decal: bool,
    pub is_fauna: bool,
    pub snow_inside: bool,
    pub snow_outside: bool,
    pub destruction_inside: bool,
    pub destruction_outside: bool,
    pub animated: bool,
    pub decal_parallax_scale: f32,
    pub decal_tiling: f32,
    pub decal_override_gbuffer_normal: bool,
    pub flags_serialise_version: u16,
    pub allow_in_outfield: bool,
    pub clamp_to_surface: bool,
    pub clamp_to_water_surface : bool,
    pub spring: bool,
    pub summer: bool,
    pub autumn: bool,
    pub winter: bool,
    pub visible_in_shroud: bool,
    pub decal_apply_to_terrain: bool,
    pub decal_apply_to_gbuffer_objects: bool,
    pub decal_render_above_snow: bool,
    pub height_mode: String,
    pub pdlc_mask: u32, //indicated whether this should only show up when a certain DLC is installed
    pub cast_shadows: bool,
    pub no_culling: bool,
}
/// END OF PROP ###############################################################

/// START OF PARTICLE_EMITTER #################################################
/// TODO:
/// ParticleEmitter
#[derive(Clone, Debug)]
pub struct ParticleEmitterList{
    pub version: u16,
    pub num_entries: u32,
    pub particle_emitter_list: Vec<ParticleEmitter>,
}
#[derive(Clone, Debug)]
pub struct ParticleEmitter{
    pub version: u16,
    pub particle_emitter_key: String,
    pub m00: f32,
    pub m01: f32,
    pub m02: f32,
    pub m10: f32,
    pub m11: f32,
    pub m12: f32,
    pub m20: f32,
    pub m21: f32,
    pub m22: f32,
    pub m30: f32,
    pub m31: f32,
    pub m32: f32,
    pub emission_rate: f32,
    pub instance_name: u16, //unsure of type, also unsure if this is actually instance_name
    pub flags_serialise_version: u16,
    pub allow_in_outfield: bool,
    pub clamp_to_surface: bool,
    pub clamp_to_water_surface: bool,
    pub spring: bool,
    pub summer: bool,
    pub autumn: bool,
    pub winter: bool,
    pub height_mode: String,
    pub pdlc_mask: u32, //indicated whether this should only show up when a certain DLC is installed
    pub autoplay: bool,
    pub visible_in_shroud: bool,
}
/// END OF PARTICLE_EMITTER ###################################################

/// START OF AI_HINTS #########################################################
/// AI_HINTS contains: AI_SEPERATORS, DIRECTED_POINTS, POLYLINES,
/// POLYLINES_LIST.
/// TODO:DirectedPointsList -> haven't been able to find an AI hint of this
/// type yet
/// NOTE: PolyLineSiegeList is listed as POLYLINES_LIST in the bmd_data, but
/// they're all siege AI hints, so I renamed it here so that it is more
/// distinct from POLYLINES. Stupid naming convention, c'mon CA that shit is
/// confusing.
#[derive(Clone, Debug)]
pub struct AiHints{
    pub version: u16,
    pub ai_separator_list: Vec<AiSeparatorList>,
    pub directed_point_list: Vec<DirectedPointsList>,
    pub polylines: Vec<PolyLines>,
    pub polylines_list: Vec<PolyLinesList>, //Not used?
}

/// TODO: Define this.
#[derive(Clone, Debug)]
pub struct PolyLinesList{
    pub nothing: Vec<u8>
}

///AI_SEPARATOR, the only type I've found thus far is PALT_FORT.
/// contains AiSeparator, AiSeparatorPoints.
#[derive(Clone, Debug)]
pub struct AiSeparatorList{
    pub version: u16,
    pub num_entries: u32,
    pub ai_separator: Vec<AiSeparator>,
}
#[derive(Clone, Debug)]
pub struct AiSeparator{
    pub version: u16,
    pub ai_separator_type: String,
    pub number_points: u32,
    pub ai_separator_points: Vec<AiSeparatorPoints>,
}
#[derive(Clone, Debug)]
pub struct AiSeparatorPoints{
    pub position_x: f32,
    pub position_y: f32,
}

/// DirectedPointsList
/// TODO: DirectedPoints is undefined.
/// This has thus far always been empty. I haven't been able to find an
/// associated AI hint yet.
#[derive(Clone, Debug)]
pub struct DirectedPointsList{
    pub version: u16,
    pub num_entries: u32,
    pub directed_points: Vec<DirectedPoints>,
}

/// TODO: Define this.
#[derive(Clone, Debug)]
pub struct DirectedPoints{
    pub nothing: Vec<u8>
}


///PolyLines, not to be confused with SiegePolyLineList, which is something else.
/// AIH_DEFENSIVE_HILL, AIH_DEFAULT_DEPLOYMENT_LINE, AIH_AMBUSH_FOREST,
/// AIH_ATTACKER_REINFORCEMENT_LINE, AIH_DEFENDER_REINFORCEMENT_LINE
#[derive(Clone, Debug)]
pub struct PolyLines{
    pub version: u16,
    pub num_entries: u32,
    pub hint_polyline: Vec<HintPolyline>,
}
#[derive(Clone, Debug)]
pub struct HintPolyline{
    pub version: u16,
    pub hint_polyline_type: String,
    pub number_of_points: u32,
    pub hint_polyline_points: Vec<HintPolylinePoints>,
}
#[derive(Clone, Debug)]
pub struct HintPolylinePoints{
    pub position_x: f32,
    pub position_y: f32,
}

///SiegePolyLines are the AI hints used for sieges:
/// AIH_SIEGE_AREA_NODE, AIH_SIEGE_INTERSECTION_NODE, AIH_SIEGE_ENTRY_NODE,
/// AIH_SIEGE_WALL_AREA_NODE, AIH_SIEGE_AREA_CONNECTION_EDGE, AIH_SIEGE_STREET_EDGE
/// AIH_SIEGE_WALL_EDGE
#[derive(Clone, Debug)]
pub struct SiegePolyLineList{
    pub version: u16,
    pub num_entries: u32,
    pub hint_polyline: Vec<SiegePolyLine>,
}
#[derive(Clone, Debug)]
pub struct SiegePolyLine{
    pub version: u16,
    pub siege_polyline_type: String,
    pub number_of_polygons: u32,
    pub siege_polyline_polygons: Vec<SiegePolyLinePolygons>,
}
#[derive(Clone, Debug)]
pub struct SiegePolyLinePolygons{
    pub number_of_points: u32,
    pub siege_polyline_points: Vec<SiegePolylinePoints>,
}
#[derive(Clone, Debug)]
pub struct SiegePolylinePoints{
    pub position_x: f32,
    pub position_y: f32,
}
/// END OF AI HINTS ###########################################################

/// START OF LIGHT_PROBE ######################################################
/// These things seem kind of useless for us, but they might be in CA maps
/// so we still need to define them I suppose.
#[derive(Clone, Debug)]
pub struct LightProbeList{
    pub version: u16,
    pub num_entries: u32,
    pub light_probe_list: Vec<LightProbe>,
}
#[derive(Clone, Debug)]
pub struct LightProbe{
    pub version: u16,
    pub position_x: f32,
    pub position_y: f32,
    pub position_z: f32,
    pub radius: f32,
    pub is_primary: bool,
    pub height_mode: String,
}
/// END OF LIGHT_PROBE ########################################################

/// START OF TERRAIN_STENCIL_TRIANGLE_LIST ####################################
/// Not to be confused with TerrainStencilBlendTriangleList
/// I have no idea what these are, but I made three of them so I should check
/// that out in Terry.
#[derive(Clone, Debug)]
pub struct TerrainStencilTriangleList{
    pub version: u16,
    pub num_entries: u32,
    pub terrain_stencil_triangle_list: Vec<TerrainStencilTriangle>,
}
#[derive(Clone, Debug)]
pub struct TerrainStencilTriangle{
    pub version: u16,
    pub pos0_x: f32,
    pub pos0_y: f32,
    pub pos0_z: f32,
    pub pos1_x: f32,
    pub pos1_y: f32,
    pub pos1_z: f32,
    pub pos2_x: f32,
    pub pos2_y: f32,
    pub pos2_z: f32,
    pub height_mode: String,
}
/// END OF TERRAIN_STENCIL_TRIANGLE_LIST ######################################

/// START OF POINT_LIGHT_LIST #################################################
/// PointLIghtList contains PointLight. This is done.
#[derive(Clone, Debug)]
pub struct PointLightList{
    pub version: u16,
    pub num_entries: u32,
    pub point_light_list: Vec<PointLight>,
}
#[derive(Clone, Debug)]
pub struct PointLight{
    pub version: u16,
    pub position_x: f32,
    pub position_y: f32,
    pub position_z: f32,
    pub radius: f32,
    pub colour_r: f32,
    pub colour_g: f32,
    pub colour_b: f32,
    pub colour_scale: f32,
    pub animation_type: u8, //this is the only u8 so far
    pub params_x: f32,
    pub params_y: f32,
    pub colour_min: f32,
    pub random_offset: f32,
    pub falloff_type: String,
    pub lf_relative: bool,
    pub height_mode: String,
    pub light_probes_only: bool,
    pub pdlc_mask: bool,
}
/// END OF POINT_LIGHT_LIST ###################################################

/// START OF BUILDING_PROJECTILE_EMITTER ######################################
/// TODO: Check building_index_number, u32 here, u16 in the actual BattleFieldBuilding
#[derive(Clone, Debug)]
pub struct BuildingProjectileEmitterList{
    pub version: u16,
    pub num_entries: u32,
    pub building_projectile_emitter_list: Vec<BuildingProjectileEmitter>,
}
#[derive(Clone, Debug)]
pub struct BuildingProjectileEmitter{
    pub version: u16,
    pub position_x: f32,
    pub position_y: f32,
    pub position_z: f32,
    pub direction_x: f32,
    pub direction_y: f32,
    pub direction_z: f32,
    pub building_index_number: u32,
    pub height_mode: String,
}
/// END OF BUILDING_PROJECTILE_EMITTER ########################################

/// START OF PLAYABLE_AREA ####################################################
/// Definition of playable area. This one is finished I think.
#[derive(Clone, Debug)]
pub struct PlayableArea{
    pub version: u16,
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
    pub has_been_set: bool,
    pub valid_location_flags_serialise_version: u16,
    pub valid_north: bool,
    pub valid_south: bool,
    pub valid_east: bool,
    pub valid_west: bool,
}
/// END OF PLAYABLE_AREA ######################################################

/// START OF CUSTOM_MATERIAL_MESH #############################################
/// CustomMaterialMesh is a polygon with a material
/// CustomMaterialMeshList contains CustomMaterialMeshList which contains
/// CustomMaterialMeshVertices and CustomMaterialMeshIndex
#[derive(Clone, Debug)]
pub struct CustomMaterialMeshList{
    pub version: u16,
    pub num_entries: u32,
    pub custom_material_mesh_list: Vec<CustomMaterialMesh>,
}
#[derive(Clone, Debug)]
pub struct CustomMaterialMesh{
    pub version: u16,
    pub num_vertices: u32,
    pub custom_material_mesh_vertices: Vec<CustomMaterialMeshVertices>,
    pub num_indices: u16,
    pub custom_material_mesh_indices: Vec<CustomMaterialMeshIndex>,
    pub material: String,
    pub height_mode: String,
}
#[derive(Clone, Debug)]
pub struct CustomMaterialMeshVertices{
    pub vertex_x: f32,
    pub vertex_y: f32,
    pub vertex_z: f32,
}
#[derive(Clone, Debug)]
pub struct CustomMaterialMeshIndex{
    pub custom_material_mesh_index: u16,
}
/// END OF CUSTOM_MATERIAL_MESH ###############################################

/// START OF TERRAIN_STENCIL_BLEND_TRIANGLE ###################################
/// Not to be confused with TerrainStencilTriangleList
/// TODO: TerrainStencilBlendTriangle - I haven't found out what this is yet
/// in Terry.
#[derive(Clone, Debug)]
pub struct TerrainStencilBlendTriangleList{
    pub version: u16,
    pub num_entries: u32,
    pub terrain_stencil_blend_triangle_list: Vec<TerrainStencilBlendTriangle>,
}
/// TODO: Define this.
#[derive(Clone, Debug)]
pub struct TerrainStencilBlendTriangle{
    pub nothing: Vec<u8>
}

/// END OF TERRAIN_STENCIL_BLEND_TRIANGLE #####################################

/// START OF SPOTLIGHT_LIST ###################################################
/// TODO: SpotLight
#[derive(Clone, Debug)]
pub struct SpotLightList{
    pub version: u16,
    pub num_entries: u32,
    pub spot_light_list: Vec<SpotLight>,
}
#[derive(Clone, Debug)]
pub struct SpotLight{
    pub version: u16,
    pub position_x: f32,
    pub position_y: f32,
    pub position_z: f32,
    pub end_i: f32,
    pub end_j: f32,
    pub end_k: f32,
    pub end_w: f32,
    pub length: f32,
    pub inner_angle: f32,
    pub outer_angle: f32,
    pub colour_r: f32,
    pub colour_g: f32,
    pub colour_b: f32,
    pub falloff: f32,
    pub gobo: u16, //unsure about the type
    pub volumetric: bool,
    pub height_mode: String,
    pub pdlc_version: u32,
}
/// END OF SPOTLIGHT_LIST #####################################################

/// START OF SOUND_SHAPE_LIST #################################################
/// There are a couple of different possible types of sound shapes:
/// SST_POINT, SST_MULTI_POINT, SST_LINE_LIST, SST_SPHERE
/// The main difference between these is the type, and the amount of points.
/// The sphere also has an outer_radius
/// The inner_cube and outer_cube seem to always be 0 for all values.
#[derive(Clone, Debug)]
pub struct SoundShapeList{
    pub version: u16,
    pub num_entries: u32,
    pub sound_shape_list: Vec<SoundShape>,
}
/// I'm very unsure of a lot of entries here, because many were 0 or empty
#[derive(Clone, Debug)]
pub struct SoundShape{
    pub version: u16,
    pub sound_shape_key: String,
    pub sound_shape_type: String,
    pub number_of_points: Vec<SoundShapePoints>,
    pub inner_radius: f32,
    pub outer_radius: f32,
    pub inner_cube_min_x: f32,
    pub inner_cube_min_y: f32,
    pub inner_cube_min_z: f32,
    pub inner_cube_max_x: f32,
    pub inner_cube_max_y: f32,
    pub inner_cube_max_z: f32,
    pub outer_cube_min_x: f32,
    pub outer_cube_min_y: f32,
    pub outer_cube_min_z: f32,
    pub outer_cube_max_x: f32,
    pub outer_cube_max_y: f32,
    pub outer_cube_max_z: f32,
    pub number_of_river_nodes: u32, //if this is not 0, we might have to add another Vector
    pub clamp_to_surface: bool,
    pub height_mode: String,
    pub campaign_type_mask: u32, //check with other campaign_type_mask entries
    pub pdlc_mask: u32,
}
///
#[derive(Clone, Debug)]
pub struct SoundShapePoints{
    pub point_x: f32,
    pub point_y: f32,
    pub point_z: f32,
}
/// END OF SOUND_SHAPE_LIST ###################################################

/// START OF COMPOSITE_SCENE ##################################################
/// TODO:
///
#[derive(Clone, Debug)]
pub struct CompositeSceneList{
    pub version: u16,
    pub num_entries: u32,
    pub composite_scene_list: Vec<CompositeScene>,
}
#[derive(Clone, Debug)]
pub struct CompositeScene{
    pub version: u16,
    pub m00: f32,
    pub m01: f32,
    pub m02: f32,
    pub m10: f32,
    pub m11: f32,
    pub m12: f32,
    pub m20: f32,
    pub m21: f32,
    pub m22: f32,
    pub m30: f32,
    pub m31: f32,
    pub m32: f32,
    pub scene_file: String,
    pub height_mode: String,
    pub pdlc_mask: u32,
    pub autoplay: bool,
    pub visible_in_shroud: bool,
    pub no_culling: bool,
}
/// END OF COMPOSITE_SCENE ####################################################

/// START OF DEPLOYMENT_AREAS #################################################
/// Deployment zones are very nested, as follows:
/// DeploymentList
/// DeploymentArea
/// DeploymentZone
/// DeploymentZoneRegion
/// DeploymentBoundaryPoints
/// This section might be done, but I'm not sure about the DeploymentBoundaryPoints
#[derive(Clone, Debug)]
pub struct DeploymentList{
    pub version: u16,
    pub num_entries: u32,
    pub sound_shape_list: Vec<DeploymentArea>,
}
#[derive(Clone, Debug)]
pub struct DeploymentArea{
    pub version: u16,
    pub category: String,
    pub num_entries: u32,
    pub deployment_area: Vec<DeploymentZone>,
}
#[derive(Clone, Debug)]
pub struct DeploymentZone{
    pub version: u16,
    pub num_entries: u32,
    pub deployment_zones: Vec<DeploymentZoneRegion>,
}
#[derive(Clone, Debug)]
pub struct DeploymentZoneRegion{
    pub version: u16,
    pub num_entries: u32,
    pub deployment_area_boundary_type: String,
    pub number_boundary_points: u16,
    pub boundary_points: Vec<DeploymentBoundaryPoints>,
    pub orientation: f32,
    pub snap_facing: bool,
    pub id: f32,
}
#[derive(Clone, Debug)]
pub struct DeploymentBoundaryPoints{
    pub position_x: f32,
    pub position_y: f32,
}
/// END OF DEPLOYMENT_AREAS ###################################################

/// START OF BMD_CATCHMENT_AREA ###############################################
/// TODO: BmdCatchmentArea
#[derive(Clone, Debug)]
pub struct BmdCatchmentAreaList{
    pub version: u16,
    pub num_entries: u32,
    pub bmd_catchment_area_list: Vec<BmdCatchmentArea>,
}

/// TODO: Define this.
#[derive(Clone, Debug)]
pub struct BmdCatchmentArea{
    pub nothing: Vec<u8>
}
// END OF BMD_CATCHMENT_AREA #################################################

/// From here, only implementations.
/// This is an example. From the program, we usually run the read function of Bmd and that one triggers
/// the decoding of every part of the file.
///
/// NOTE: the decoding stuff is error-prone. I haven't yet properly make it check for errors, so if you
/// provide it with the wrong type of data, it will crash the program.
///
/// NOTE2: If you want to see what data are you using, use:
/// println!("{:?}", variable_you_want_to_see);
/// That will print in the "Run" terminal what data it's in the variable.
impl Bmd {
    pub fn read(packed_file_data: Vec<u8>) -> Bmd {
        let packed_file_header = BmdHeader::read(packed_file_data[..8].to_vec());
        let packed_file_data = BmdData::read(packed_file_data[8..].to_vec());
        Bmd {
            packed_file_header,
            packed_file_data
        }
    }
}

/// Example. It needs to have proper decoding for both variables.
impl BmdHeader {
    pub fn read(packed_file_data: Vec<u8>) -> BmdHeader {
        let bmd_file_header_bmd_file_type = format!("header type");
        let bmd_file_header_bmd_file_version = 0u16;
        BmdHeader {
            bmd_file_header_bmd_file_type,
            bmd_file_header_bmd_file_version,
        }
    }
}

/// Example. It needs to have proper decoding for all variables.
impl BmdData {
    pub fn read(packed_file_data: Vec<u8>) -> BmdData {
        let battlefield_building_list = vec![];
        let battlefield_building_list_far = vec![];
        let capture_location_set = vec![];
        let ef_line_list = vec![];
        let go_outlines = vec![];
        let non_terrain_outlines = vec![];
        let zones_template_list = vec![];
        let prefab_instance_list = vec![];
        let bmd_outline_list = vec![];
        let terrain_outlines = vec![];
        let lite_building_outlines = vec![];
        let camera_zones = vec![];
        let civilian_deployment_list = vec![];
        let civilian_shelter_list = vec![];
        let prop_list = vec![];
        let particle_emitter_list = vec![];
        let ai_hints = vec![];
        let light_probe_list = vec![];
        let terrain_stencil_triangle_list = vec![];
        let point_light_list = vec![];
        let building_projectile_emitter_list = vec![];
        let playable_area = vec![];
        let custom_material_mesh_list = vec![];
        let terrain_stencil_blend_triangle_list = vec![];
        let spot_light_list = vec![];
        let sound_shape_list = vec![];
        let composite_scene_list = vec![];
        let deployment_list = vec![];
        let bmd_catchment_area_list = vec![];
        BmdData {
            battlefield_building_list,
            battlefield_building_list_far,
            capture_location_set,
            ef_line_list,
            go_outlines,
            non_terrain_outlines,
            zones_template_list,
            prefab_instance_list,
            bmd_outline_list,
            terrain_outlines,
            lite_building_outlines,
            camera_zones,
            civilian_deployment_list,
            civilian_shelter_list,
            prop_list,
            particle_emitter_list,
            ai_hints,
            light_probe_list,
            terrain_stencil_triangle_list,
            point_light_list,
            building_projectile_emitter_list,
            playable_area,
            custom_material_mesh_list,
            terrain_stencil_blend_triangle_list,
            spot_light_list,
            sound_shape_list,
            composite_scene_list,
            deployment_list,
            bmd_catchment_area_list,
        }
    }
}