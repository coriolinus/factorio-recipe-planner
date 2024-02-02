use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationSecondsWithFrac, Seq};
use time::Duration;

const HALF_SECOND: Duration = Duration::milliseconds(500);

fn true_() -> bool {
    true
}

fn is_true(v: &bool) -> bool {
    *v
}

const fn half_second() -> Duration {
    HALF_SECOND
}

fn is_half_second(v: &Duration) -> bool {
    *v == HALF_SECOND
}

fn one_u8() -> u8 {
    1
}

fn is_one_u8(v: &u8) -> bool {
    *v == 1
}

fn one_f64() -> f64 {
    1.0
}

fn is_one_f64(v: &f64) -> bool {
    *v == 1.0
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IconData {
    #[serde(rename = "icon")]
    pub path: PathBuf,
    #[serde(rename = "icon_mipmaps")]
    pub mipmaps: u8,
    #[serde(rename = "icon_size")]
    pub size: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SingleOutput {
    #[serde(rename = "result")]
    pub name: String,
    #[serde(
        rename = "result_count",
        default = "one_u8",
        skip_serializing_if = "is_one_u8"
    )]
    pub amount: u8,
    #[serde(
        rename = "result_probability",
        default = "one_f64",
        skip_serializing_if = "is_one_f64"
    )]
    pub probability: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutputItem {
    pub name: String,
    pub amount: u8,
    #[serde(default = "one_f64", skip_serializing_if = "is_one_f64")]
    pub probability: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ManyOutputs {
    #[serde(rename = "results")]
    pub outputs: Vec<OutputItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Output {
    Single(SingleOutput),
    Many(ManyOutputs),
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Recipe {
    pub name: String,
    pub r#type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subgroup: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,
    #[serde(default = "true_", skip_serializing_if = "is_true")]
    pub enabled: bool,
    #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<IconData>,
    #[serde_as(as = "Seq<(_, _)>")]
    pub ingredients: HashMap<String, u8>,
    #[serde(flatten)]
    pub output: Output,
    #[serde_as(as = "DurationSecondsWithFrac<f64>")]
    #[serde(
        rename = "energy_required",
        default = "half_second",
        skip_serializing_if = "is_half_second"
    )]
    pub duration: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(recipe: &str) {
        let parsed = serde_json::from_str::<Recipe>(recipe).unwrap();
        let value = serde_json::from_str::<serde_json::Value>(recipe).unwrap();
        let reserialized = serde_json::to_string(&parsed).unwrap();
        let value_of_reserialized =
            serde_json::from_str::<serde_json::Value>(&reserialized).unwrap();

        println!("recipe:\n{recipe}");
        println!("reserialized:\n{reserialized}");

        assert_eq!(value_of_reserialized, value);
    }
    mod copper_cable {
        use super::super::*;
        use maplit::hashmap;

        const RECIPE: &str = r#"{"ingredients":[["copper-plate",1]],"name":"copper-cable","result":"copper-cable","result_count":2,"type":"recipe"}"#;

        #[test]
        fn parses() {
            let expect = Recipe {
                name: "copper-cable".into(),
                r#type: "recipe".into(),
                category: None,
                subgroup: None,
                order: None,
                enabled: true,
                icon: None,
                ingredients: hashmap! {"copper-plate".into() => 1},
                output: Output::Single(SingleOutput {
                    name: "copper-cable".into(),
                    amount: 2,
                    probability: 1.0,
                }),
                duration: Duration::milliseconds(500),
            };

            let found = serde_json::from_str::<Recipe>(RECIPE).unwrap();
            assert_eq!(found, expect);
        }

        #[test]
        fn roundtrip() {
            super::roundtrip(RECIPE);
        }
    }

    mod uranium_processing {
        use super::super::*;
        use maplit::hashmap;

        // Note that this is not quite the recipe from the real recipe book;
        // this expresses the energy required as a float, where the recipe book
        // has it as an integer.
        //
        // This is necessary for the round-trip test to work, because our data model converts it to a float in all cases.
        const RECIPE: &str = r#"{"category":"centrifuging","enabled":false,"energy_required":12.0,"icon":"__base__/graphics/icons/uranium-processing.png","icon_mipmaps":4,"icon_size":64,"ingredients":[["uranium-ore",10]],"name":"uranium-processing","order":"k[uranium-processing]","results":[{"amount":1,"name":"uranium-235","probability":0.007000000000000001},{"amount":1,"name":"uranium-238","probability":0.993}],"subgroup":"raw-material","type":"recipe"}"#;

        #[test]
        fn parses() {
            let expect = Recipe {
                name: "uranium-processing".into(),
                r#type: "recipe".into(),
                category: Some("centrifuging".into()),
                subgroup: Some("raw-material".into()),
                order: Some("k[uranium-processing]".into()),
                enabled: false,
                icon: Some(IconData {
                    path: "__base__/graphics/icons/uranium-processing.png".into(),
                    mipmaps: 4,
                    size: 64,
                }),
                ingredients: hashmap! {"uranium-ore".into() => 10},
                output: Output::Many(ManyOutputs {
                    outputs: vec![
                        OutputItem {
                            name: "uranium-235".into(),
                            amount: 1,
                            probability: 0.007000000000000001,
                        },
                        OutputItem {
                            name: "uranium-238".into(),
                            amount: 1,
                            probability: 0.993,
                        },
                    ],
                }),
                duration: Duration::seconds(12),
            };

            let found = serde_json::from_str::<Recipe>(RECIPE).unwrap();
            assert_eq!(found, expect);
        }

        #[test]
        fn roundtrip() {
            super::roundtrip(RECIPE);
        }
    }
}
