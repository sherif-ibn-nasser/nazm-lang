use nazmc_nir::{PoolIdx, Span};
use nazmc_resolve::FileItemKindAndIdx;
use thin_vec::ThinVec;

#[derive(Default)]
pub(crate) struct NIRBuilderErrors {
    pkg_paths_errs: Vec<PkgPathErr>,
    unresolved_item_errs: Vec<UnresolvedItemErr>,
    encapsulation_errs: Vec<EncapsulationErr>,
    wrong_file_item_found: Vec<WrongFileItemFoundErr>,
}

impl NIRBuilderErrors {
    pub(crate) fn report_pkg_path_err(
        &mut self,
        file_idx: usize,
        pkg_path: ThinVec<PoolIdx>,
        pkg_path_spans: ThinVec<Span>,
    ) {
        self.pkg_paths_errs.push(PkgPathErr {
            file_idx,
            pkg_path,
            pkg_path_spans,
        });
    }

    pub(crate) fn report_unresolved_item(&mut self, file_idx: usize, id: PoolIdx, span: Span) {
        self.unresolved_item_errs
            .push(UnresolvedItemErr { file_idx, id, span });
    }

    pub(crate) fn report_encapsulation_err(
        &mut self,
        file_idx: usize,
        resolved_file_idx: usize,
        resolved_item_idx: usize,
        id: PoolIdx,
        span: Span,
    ) {
        self.encapsulation_errs.push(EncapsulationErr {
            file_idx,
            resolved_file_idx,
            resolved_item_idx,
            id,
            span,
        });
    }

    pub(crate) fn report_wrong_file_item_found_err(
        &mut self,
        file_idx: usize,
        span: Span,
        found_kind: u64,
    ) {
        self.wrong_file_item_found.push(WrongFileItemFoundErr {
            file_idx,
            span,
            found_kind,
        });
    }
}

pub(crate) struct PkgPathErr {
    file_idx: usize,
    pkg_path: ThinVec<PoolIdx>,
    pkg_path_spans: ThinVec<Span>,
}

pub(crate) struct UnresolvedItemErr {
    file_idx: usize,
    id: PoolIdx,
    span: Span,
}

pub(crate) struct EncapsulationErr {
    file_idx: usize,
    resolved_file_idx: usize,
    resolved_item_idx: usize,
    id: PoolIdx,
    span: Span,
}

pub(crate) struct WrongFileItemFoundErr {
    file_idx: usize,
    span: Span,
    found_kind: u64,
}
