use crate::core::source::MaybePackage;
use crate::core::{Dependency, Package, PackageId, Source, SourceId, Summary};
use crate::util::errors::CargoResult;

pub struct StdlibSource {
    source_id: SourceId,
}

impl StdlibSource {
    pub fn new(source_id: SourceId) -> StdlibSource {
        StdlibSource { source_id }
    }
}

impl Source for StdlibSource {
    fn source_id(&self) -> SourceId {
        self.source_id
    }

    fn supports_checksums(&self) -> bool {
        false
    }

    fn requires_precise(&self) -> bool {
        false
    }

    fn query(&mut self, _dep: &Dependency, _f: &mut dyn FnMut(Summary)) -> CargoResult<()> {
        unimplemented!()
    }

    fn fuzzy_query(&mut self, _dep: &Dependency, _f: &mut dyn FnMut(Summary)) -> CargoResult<()> {
        unimplemented!()
    }

    fn update(&mut self) -> CargoResult<()> {
        Ok(())
    }

    fn download(&mut self, _package: PackageId) -> CargoResult<MaybePackage> {
        unimplemented!()
    }

    fn finish_download(&mut self, _package: PackageId, _contents: Vec<u8>) -> CargoResult<Package> {
        unimplemented!()
    }

    fn fingerprint(&self, _pkg: &Package) -> CargoResult<String> {
        unimplemented!()
    }

    fn describe(&self) -> String {
        "stdlib source".to_string()
    }

    fn add_to_yanked_whitelist(&mut self, _pkgs: &[PackageId]) {}

    fn is_yanked(&mut self, _pkg: PackageId) -> CargoResult<bool> {
        Ok(false)
    }
}
