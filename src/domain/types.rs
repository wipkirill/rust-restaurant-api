use chrono::{Duration, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, AddAssign};
use unicode_segmentation::UnicodeSegmentation;

// Contains all structs and traits to satisfy all validation constraints

// Id type for TableId and ItemId
pub type IdType = u32;
// A type for order quantity
pub type QuantityType = u32;
// A type for item version
pub type VersionType = u32;

#[derive(PartialEq, Hash, Eq, Debug, Serialize, Deserialize, Copy, Clone)]
pub struct ItemId<T>(T);

impl TryFrom<String> for ItemId<IdType> {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let is_numeric = s.parse::<IdType>().is_ok();
        let mut is_greater_zero = true;
        if is_numeric {
            let val = s.parse::<IdType>().unwrap();
            is_greater_zero = val > 0;
        }
        if is_numeric && is_greater_zero {
            Ok(Self(s.parse::<IdType>().unwrap()))
        } else {
            Err(format!("'{}' is not a valid item id.", s))
        }
    }
}

impl From<ItemId<IdType>> for IdType {
    fn from(value: ItemId<IdType>) -> Self {
        value.0
    }
}

impl fmt::Display for ItemId<IdType> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct ItemName(String);
impl TryFrom<String> for ItemName {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let is_empty_or_whitespace = s.trim().is_empty();
        let is_too_long = s.graphemes(true).count() > 100;

        // Iterate over all characters in the input `s` to check if any of them matches
        // one of the characters in the forbidden array.
        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
            Err(format!("'{}' is not a valid item name.", s))
        } else {
            Ok(Self(s))
        }
    }
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct ItemNotes(String);
impl TryFrom<String> for ItemNotes {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let is_too_long = s.graphemes(true).count() > 256;

        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));

        if is_too_long || contains_forbidden_characters {
            Err(format!("{} is not a valid notes.", s))
        } else {
            Ok(Self(s))
        }
    }
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct ItemQuantity<T>(T);

impl TryFrom<String> for ItemQuantity<QuantityType> {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let is_numeric = s.parse::<IdType>().is_ok();

        if is_numeric {
            Ok(Self(s.parse::<QuantityType>().unwrap()))
        } else {
            Err(format!("'{}' is not a valid quantity value.", s))
        }
    }
}

impl From<ItemQuantity<QuantityType>> for QuantityType {
    fn from(value: ItemQuantity<QuantityType>) -> Self {
        value.0
    }
}

#[derive(Debug, Serialize, Clone, PartialEq, PartialOrd)]
pub struct ItemVersion<T>(T);

impl TryFrom<String> for ItemVersion<VersionType> {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let is_numeric = s.parse::<VersionType>().is_ok();

        if is_numeric {
            Ok(Self(s.parse::<VersionType>().unwrap()))
        } else {
            Err(format!("'{}' is not a valid version number.", s))
        }
    }
}

impl From<ItemVersion<VersionType>> for VersionType {
    fn from(value: ItemVersion<VersionType>) -> Self {
        value.0
    }
}

impl AddAssign for ItemVersion<VersionType> {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0
    }
}

impl fmt::Display for ItemVersion<VersionType> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Add for ItemVersion<VersionType> {
    type Output = Self;
    fn add(self, other: ItemVersion<u32>) -> <Self as Add<ItemVersion<u32>>>::Output {
        Self(self.0 + other.0)
    }
}

impl ItemVersion<VersionType> {
    pub fn from_int(number: u32) -> Self {
        Self(number)
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Item {
    pub id: ItemId<IdType>,
    pub name: ItemName,
    pub notes: ItemNotes,
    pub quantity: ItemQuantity<QuantityType>,
    pub deleted: bool,
    pub version: ItemVersion<VersionType>,
    pub time_to_prepare: String,
}

impl Item {
    pub fn new(
        item_id: ItemId<IdType>,
        item_name: ItemName,
        item_notes: ItemNotes,
        item_quantity: ItemQuantity<QuantityType>,
        item_deleted: bool,
        item_version: ItemVersion<VersionType>,
        item_time_to_prepare: String,
    ) -> Self {
        Self {
            id: item_id,
            name: item_name,
            notes: item_notes,
            quantity: item_quantity,
            deleted: item_deleted,
            version: item_version,
            time_to_prepare: item_time_to_prepare,
        }
    }
    pub fn gen_time_to_prepare(&mut self) {
        let mut rng = rand::thread_rng();
        self.time_to_prepare = (Utc::now() + Duration::minutes(rng.gen_range(1..16))).to_string();
    }
}

#[derive(PartialEq, Hash, Eq, Debug, Serialize, Deserialize, Copy, Clone)]
pub struct TableId<T>(T);

impl TryFrom<String> for TableId<IdType> {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let is_numeric = s.parse::<IdType>().is_ok();
        let mut is_in_range = true;
        if is_numeric {
            let val = s.parse::<IdType>().unwrap();
            let allowed_table_id_range = 1..101;
            is_in_range = allowed_table_id_range.contains(&val);
        }
        if is_numeric && is_in_range {
            Ok(Self(s.parse::<IdType>().unwrap()))
        } else {
            Err(format!("{} is not a valid table id.", s))
        }
    }
}

impl From<TableId<IdType>> for IdType {
    fn from(value: TableId<IdType>) -> Self {
        value.0
    }
}

impl fmt::Display for TableId<IdType> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
impl ItemId<IdType> {
    pub fn id_one() -> Self {
        Self(1)
    }
    pub fn from_int(number: u32) -> Self {
        Self(number)
    }
}

#[cfg(test)]
impl TableId<IdType> {
    pub fn id_one() -> Self {
        Self(1)
    }
    pub fn from_int(number: u32) -> Self {
        Self(number)
    }
}

impl From<ItemName> for String {
    fn from(n: ItemName) -> Self {
        n.0
    }
}

#[cfg(test)]
impl ItemName {
    pub fn pizza() -> Self {
        Self(String::from("Some pizza"))
    }
    pub fn pasta() -> Self {
        Self(String::from("Some pasta"))
    }
    pub fn bad() -> Self {
        Self(String::from(""))
    }
    pub fn from_str(s: String) -> Self {
        Self(s)
    }
}

impl From<ItemNotes> for String {
    fn from(n: ItemNotes) -> Self {
        n.0
    }
}

#[cfg(test)]
impl ItemNotes {
    pub fn some_notes() -> Self {
        Self(String::from("Some notes"))
    }
    pub fn other_notes() -> Self {
        Self(String::from("Some other notes"))
    }
    pub fn from_str(s: String) -> Self {
        Self(s)
    }
}

#[cfg(test)]
impl ItemQuantity<QuantityType> {
    pub fn one() -> Self {
        Self(1)
    }
    pub fn two() -> Self {
        Self(1)
    }
    pub fn from_int(number: u32) -> Self {
        Self(number)
    }
}

#[cfg(test)]
impl ItemVersion<VersionType> {
    pub fn ver_one() -> Self {
        Self(1)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::types::{ItemId, ItemName, ItemNotes, TableId};
    use claim::{assert_err, assert_ok};

    #[test]
    fn a_100_grapheme_long_name_is_valid() {
        let name = "ё".repeat(100);
        assert_ok!(ItemName::try_from(name));
    }

    #[test]
    fn a_name_longer_than_100_graphemes_is_rejected() {
        let name = "a".repeat(257);
        assert_err!(ItemName::try_from(name));
    }

    #[test]
    fn whitespace_only_names_are_rejected() {
        let name = " ".to_string();
        assert_err!(ItemName::try_from(name));
    }

    #[test]
    fn empty_string_is_rejected() {
        let name = "".to_string();
        assert_err!(ItemName::try_from(name));
    }

    #[test]
    fn names_containing_an_invalid_character_are_rejected() {
        for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            let name = name.to_string();
            assert_err!(ItemName::try_from(name));
        }
    }

    #[test]
    fn a_valid_name_is_parsed_successfully() {
        let name = "Hamburger".to_string();
        assert_ok!(ItemName::try_from(name));
    }

    #[test]
    fn a_256_grapheme_long_name_is_valid() {
        let name = "A".repeat(256);
        assert_ok!(ItemNotes::try_from(name));
    }

    #[test]
    fn a_name_longer_than_256_graphemes_is_rejected() {
        let name = "a".repeat(257);
        assert_err!(ItemNotes::try_from(name));
    }

    #[test]
    fn notes_containing_an_invalid_character_are_rejected() {
        for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            let name = name.to_string();
            assert_err!(ItemNotes::try_from(name));
        }
    }

    #[test]
    fn a_valid_note_is_parsed_successfully() {
        let name = "Some notes".to_string();
        assert_ok!(ItemName::try_from(name));
    }

    #[test]
    fn a_valid_table_id_is_parsed_successfully() {
        let table_id = "1".to_string();
        assert_ok!(TableId::try_from(table_id));
    }

    #[test]
    fn an_invalid_table_id_is_rejected() {
        let mut table_id = "-1".to_string();
        assert_err!(TableId::try_from(table_id));
        table_id = "0".to_string();
        assert_err!(TableId::try_from(table_id));
        table_id = "200".to_string();
        assert_err!(TableId::try_from(table_id));
    }

    #[test]
    fn a_valid_item_id_is_parsed_successfully() {
        let item_id = "1".to_string();
        assert_ok!(ItemId::try_from(item_id));
    }

    #[test]
    fn an_invalid_item_id_is_rejected() {
        let mut item_id = "-1".to_string();
        assert_err!(ItemId::try_from(item_id));
        item_id = "0".to_string();
        assert_err!(ItemId::try_from(item_id));
    }
}
