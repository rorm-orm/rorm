use rorm_macro::impl_annotations_builder;

use rorm_declaration::hmr::annotations;

impl_annotations_builder!(
    auto_create_time AutoCreateTime,
    auto_update_time AutoUpdateTime,
    auto_increment   AutoIncrement,
    choices          Choices,
    default          DefaultValue,
    index            Index,
    max_length       MaxLength,
    primary_key      PrimaryKey,
    unique           Unique,
);

/// Helper trait to propagate `IMPLICIT_NOT_NULL` from [Annotation] to [Annotations]
///
/// [Annotation]: annotations::Annotation
pub trait ImplicitNotNull {
    /// `true`, if any of the annotations' `IMPLICIT_NOT_NULL` is true.
    const IMPLICIT_NOT_NULL: bool;
}

/// The resulting type, when adding an annotation `T` to an [`Annotations`] struct `A`
///
/// ```
/// use rorm_declaration::hmr::annotations::Unique;
/// use rorm::annotation_builder::{Add, NotSetAnnotations};
/// let _: Add<Unique, NotSetAnnotations> = NotSetAnnotations::new().unique(Unique);
/// ```
pub type Add<T, A> = <A as annotations::Step<T>>::Output;

/// Alias for `Add<Implicit<T>, A>`
///
/// Further reading:
/// - [`Add`]
/// - [`Implicit`]
///
/// ```
/// use rorm_declaration::hmr::annotations::Unique;
/// use rorm::annotation_builder::{Implicit, NotSetAnnotations};
/// let _: Implicit<Unique, NotSetAnnotations> = NotSetAnnotations::new().implicit_unique(Unique);
/// ```
///
/// [`Implicit`]: annotations::Implicit
pub type Implicit<T, A> = <A as annotations::Step<annotations::Implicit<T>>>::Output;

/// Alias for `Add<Forbidden<T>, A>`
///
/// Further reading:
/// - [`Add`]
/// - [`Forbidden`]
///
/// ```
/// use rorm_declaration::hmr::annotations::Unique;
/// use rorm::annotation_builder::{Forbidden, NotSetAnnotations};
/// let _: Forbidden<Unique, NotSetAnnotations> = NotSetAnnotations::new().forbidden_unique();
/// ```
///
/// [`Forbidden`]: annotations::Forbidden
pub type Forbidden<T, A> = <A as annotations::Step<annotations::Forbidden<T>>>::Output;
