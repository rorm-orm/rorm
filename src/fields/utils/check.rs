//! Re-usable implementations of [`FieldType::Check`]

use crate::const_fn;
#[cfg(doc)]
use crate::fields::traits::FieldType;
use crate::internal::const_concat::ConstString;
use crate::internal::hmr::annotations::Annotations;

const_fn! {
    /// [`FieldType::Check`] which checks the explicit annotations to be empty.
    pub fn disallow_annotations_check<const N: usize>(field: Annotations, _columns: [Annotations; N]) -> Result<(), ConstString<1024>> {
        match field {
            Annotations {
                auto_create_time: None,
                auto_update_time: None,
                auto_increment: None,
                choices: None,
                default: None,
                index: None,
                max_length: None,
                on_delete: None,
                on_update: None,
                primary_key: None,
                unique: None,
                nullable: false,
                foreign: None,
            } => Ok(()),
            _ => Err(ConstString::error(&["BackRef doesn't take any annotations"])),
        }
    }
}

const_fn! {
    /// [`FieldType::Check`] which runs the linter shared with `rorm-cli` on every column.
    pub fn shared_linter_check<const N: usize>(_field: Annotations, columns: [Annotations; N]) -> Result<(), ConstString<1024>> {
        let mut columns = columns.as_slice();
        while let [column, tail @ ..] = columns {
            columns = tail;

            if let Err(err) = column.as_lint().check() {
                return Err(ConstString::error(&["invalid annotations: ", err]));
            }
            if column.primary_key.is_some() && column.nullable {
                return Err(ConstString::error(&["invalid annotations: primary_key can't be Option"]));
            }
        }
        Ok(())
    }
}

const_fn! {
    /// [`FieldType::Check`] which runs the linter shared with `rorm-cli` on every column
    /// and checks `max_length` to be set.
    pub fn string_check(_field: Annotations, [column]: [Annotations; 1]) -> Result<(), ConstString<1024>> {
        if let Err(error) = shared_linter_check(_field, [column]) {
            return Err(error);
        }

        if column.max_length.is_none() {
            return Err(ConstString::error(&[
                "missing annotation: max_length",
            ]));
        }

        Ok(())
    }
}
