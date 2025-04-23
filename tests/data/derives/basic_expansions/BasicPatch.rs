#[doc(hidden)]
#[allow(non_camel_case_types)]
pub enum __BasicPatch_ValueSpaceImpl {
    BasicPatch,
    #[allow(dead_code)]
    #[doc(hidden)]
    __BasicPatch_ValueSpaceImplMarker(::std::marker::PhantomData<BasicPatch>),
}
pub use __BasicPatch_ValueSpaceImpl::*;
#[doc(hidden)]
pub struct __BasicPatch_Decoder {}
impl ::rorm::crud::selector::Selector for __BasicPatch_ValueSpaceImpl {
    type Result = BasicPatch;
    type Model = BasicModel;
    type Decoder = __BasicPatch_Decoder;
    const INSERT_COMPATIBLE: bool = true;
    fn select(
        self,
        ctx: &mut ::rorm::internal::query_context::QueryContext,
    ) -> Self::Decoder {
        __BasicPatch_Decoder {}
    }
}
impl ::std::default::Default for __BasicPatch_ValueSpaceImpl {
    fn default() -> Self {
        Self::BasicPatch
    }
}
impl ::rorm::crud::decoder::Decoder for __BasicPatch_Decoder {
    type Result = BasicPatch;
    fn by_name<'index>(
        &'index self,
        row: &'_ ::rorm::db::Row,
    ) -> Result<Self::Result, ::rorm::db::row::RowError<'index>> {
        Ok(BasicPatch {})
    }
    fn by_index<'index>(
        &'index self,
        row: &'_ ::rorm::db::Row,
    ) -> Result<Self::Result, ::rorm::db::row::RowError<'index>> {
        Ok(BasicPatch {})
    }
}
impl ::rorm::model::Patch for BasicPatch {
    type Model = BasicModel;
    type ValueSpaceImpl = __BasicPatch_ValueSpaceImpl;
    fn push_columns(columns: &mut Vec<::rorm::fields::utils::column_name::ColumnName>) {}
    fn push_references<'a>(&'a self, values: &mut Vec<::rorm::conditions::Value<'a>>) {}
    fn push_values(self, values: &mut Vec<::rorm::conditions::Value>) {}
}
impl<'a> ::rorm::internal::patch::IntoPatchCow<'a> for BasicPatch {
    type Patch = BasicPatch;
    fn into_patch_cow(self) -> ::rorm::internal::patch::PatchCow<'a, BasicPatch> {
        ::rorm::internal::patch::PatchCow::Owned(self)
    }
}
impl<'a> ::rorm::internal::patch::IntoPatchCow<'a> for &'a BasicPatch {
    type Patch = BasicPatch;
    fn into_patch_cow(self) -> ::rorm::internal::patch::PatchCow<'a, BasicPatch> {
        ::rorm::internal::patch::PatchCow::Borrowed(self)
    }
}
