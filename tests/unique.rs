use rorm::imr::Index;
use rorm::model::Model;
use rorm_macro::{FieldType, Model};

#[derive(Model)]
#[rorm(unique(owner, name), unique(position))]
#[rorm(unique(name, owner))]
#[allow(
    clippy::duplicated_attributes,
    reason = "clippy mistakes a field being part of two uniques for a duplicated attribute"
)]
struct Item {
    #[rorm(id)]
    id: i64,
    owner: i64,
    #[rorm(max_length = 255, rename = "title")]
    name: String,
    position: Position,
}

#[derive(FieldType)]
#[rorm(experimental)]
struct Position {
    x: i16,
    y: i16,
}

fn unique(name: &str, columns: &[&str]) -> Index {
    Index {
        name: Some(name.to_string()),
        columns: columns.iter().map(|column| column.to_string()).collect(),
        unique: true,
    }
}

#[test]
fn unique_spans_its_fields_columns() {
    assert_eq!(
        Item::get_imr().indexes(),
        vec![
            unique("owner_title", &["owner", "title"]),
            unique("title_owner", &["title", "owner"]),
            unique("position", &["position_x", "position_y"]),
        ]
    );
}
