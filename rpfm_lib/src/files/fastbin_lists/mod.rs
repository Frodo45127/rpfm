//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Lists of a battle map saved as their own FASTBIN0 files, as Attila's tiles have them.
//!
//! | File                     | Type                       | Contents                           |
//! |--------------------------|----------------------------|------------------------------------|
//! | `*.tree_list.bin`        | [`TreeListFile`]           | Trees, by model                    |
//! | `*.grass_list.bin`       | [`GrassListFile`]          | Grass, by type                     |
//! | `civilian_deployment.bin`| [`CivilianDeploymentFile`] | Areas where civilians are deployed |
//! | `civilian_shelter.bin`   | [`CivilianShelterFile`]    | Places where civilians shelter     |
//!
//! Each file is the FASTBIN0 signature, the version of the list, and the list.

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};
use crate::files::bmd::civilian_deployment_list::CivilianDeploymentList;
use crate::files::bmd::civilian_shelter_list::CivilianShelterList;
use crate::files::bmd_vegetation::grass_list::GrassList;
use crate::files::bmd_vegetation::tree_list::TreeList;
use crate::utils::check_size_mismatch;

/// End of the paths of tree list files.
pub const TREE_LIST_EXTENSION: &str = ".tree_list.bin";

/// End of the paths of grass list files.
pub const GRASS_LIST_EXTENSION: &str = ".grass_list.bin";

/// Name of civilian deployment files.
pub const CIVILIAN_DEPLOYMENT_FILE_NAME: &str = "civilian_deployment.bin";

/// Name of civilian shelter files.
pub const CIVILIAN_SHELTER_FILE_NAME: &str = "civilian_shelter.bin";

/// FASTBIN0 file signature.
pub const SIGNATURE: &[u8; 8] = b"FASTBIN0";

/// Version of the civilian list files. No other is known.
const CIVILIAN_LIST_VERSION: u16 = 1;

#[cfg(test)] mod fastbin_lists_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// A tree list file. The list has its version.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct TreeListFile {
    tree_list: TreeList,
}

/// A grass list file. The list has its version.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct GrassListFile {
    grass_list: GrassList,
}

/// A civilian deployment file.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct CivilianDeploymentFile {
    serialise_version: u16,
    civilian_deployment_list: CivilianDeploymentList,
}

/// A civilian shelter file.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct CivilianShelterFile {
    serialise_version: u16,
    civilian_shelter_list: CivilianShelterList,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl Decodeable for TreeListFile {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        read_signature(data)?;
        let tree_list = TreeList::decode(data, extra_data)?;
        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;

        Ok(Self { tree_list })
    }
}

impl Encodeable for TreeListFile {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_all(SIGNATURE)?;
        self.tree_list.encode(buffer, extra_data)
    }
}

impl Decodeable for GrassListFile {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        read_signature(data)?;
        let grass_list = GrassList::decode(data, extra_data)?;
        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;

        Ok(Self { grass_list })
    }
}

impl Encodeable for GrassListFile {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_all(SIGNATURE)?;
        self.grass_list.encode(buffer, extra_data)
    }
}

impl Decodeable for CivilianDeploymentFile {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        read_signature(data)?;
        let serialise_version = read_civilian_list_version(data, "CivilianDeploymentFile")?;
        let civilian_deployment_list = CivilianDeploymentList::decode(data, extra_data)?;
        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;

        Ok(Self { serialise_version, civilian_deployment_list })
    }
}

impl Encodeable for CivilianDeploymentFile {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_all(SIGNATURE)?;
        buffer.write_u16(self.serialise_version)?;
        self.civilian_deployment_list.encode(buffer, extra_data)
    }
}

impl Decodeable for CivilianShelterFile {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        read_signature(data)?;
        let serialise_version = read_civilian_list_version(data, "CivilianShelterFile")?;
        let civilian_shelter_list = CivilianShelterList::decode(data, extra_data)?;
        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;

        Ok(Self { serialise_version, civilian_shelter_list })
    }
}

impl Encodeable for CivilianShelterFile {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_all(SIGNATURE)?;
        buffer.write_u16(self.serialise_version)?;
        self.civilian_shelter_list.encode(buffer, extra_data)
    }
}

/// Reads the FASTBIN0 signature, failing if it's not there.
fn read_signature<R: ReadBytes>(data: &mut R) -> Result<()> {
    let signature = data.read_slice(8, false)?;
    if signature.as_slice() != SIGNATURE {
        return Err(RLibError::DecodingFastBinUnsupportedSignature(signature));
    }

    Ok(())
}

/// Reads the version of a civilian list file, failing if it's not the known one.
fn read_civilian_list_version<R: ReadBytes>(data: &mut R, file_type: &str) -> Result<u16> {
    let version = data.read_u16()?;
    if version != CIVILIAN_LIST_VERSION {
        return Err(RLibError::DecodingFastBinUnsupportedVersion(file_type.to_owned(), version));
    }

    Ok(version)
}
