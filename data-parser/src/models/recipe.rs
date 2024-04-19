use std::path::PathBuf;

use monostate::MustBe;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationSecondsWithFrac};
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fluid {
    pub amount: u32,
    pub name: String,
    r#type: MustBe!("fluid"),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub amount: u32,
    pub name: String,
    r#type: MustBe!("item"),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Ingredient {
    SimpleItem(String, u32),
    Fluid(Fluid),
    Item(Item),
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecipeData {
    #[serde(default = "true_", skip_serializing_if = "is_true")]
    pub enabled: bool,
    #[serde_as(as = "DurationSecondsWithFrac<f64>")]
    #[serde(
        rename = "energy_required",
        default = "half_second",
        skip_serializing_if = "is_half_second"
    )]
    pub duration: Duration,
    pub ingredients: Vec<Ingredient>,
    #[serde(flatten)]
    pub output: Output,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecipeDataWithHardMode {
    normal: RecipeData,
    expensive: RecipeData,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RecipeDataEnum {
    Simple(RecipeData),
    WithHardMode(RecipeDataWithHardMode),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Recipe {
    pub name: String,
    pub r#type: MustBe!("recipe"),
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subgroup: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,
    #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<IconData>,
    #[serde(flatten)]
    pub recipe_data: RecipeDataEnum,
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
    mod iron_stick {
        use super::super::*;

        const RECIPE: &str = r#"{"ingredients":[["iron-plate",1]],"name":"iron-stick","result":"iron-stick","result_count":2,"type":"recipe"}"#;

        #[test]
        fn parses() {
            let expect = Recipe {
                r#type: MustBe!("recipe"),
                name: "iron-stick".into(),
                category: None,
                subgroup: None,
                order: None,
                recipe_data: RecipeDataEnum::Simple(RecipeData {
                    enabled: true,
                    ingredients: vec![Ingredient::SimpleItem("iron-plate".into(), 1)],
                    output: Output::Single(SingleOutput {
                        name: "iron-stick".into(),
                        amount: 2,
                        probability: 1.0,
                    }),
                    duration: Duration::milliseconds(500),
                }),
                icon: None,
            };

            let found = serde_json::from_str::<Recipe>(RECIPE).unwrap();
            assert_eq!(found, expect);
        }

        #[test]
        fn roundtrip() {
            super::roundtrip(RECIPE);
        }
    }
    mod copper_cable {
        use super::super::*;

        const RECIPE: &str = r#"{"ingredients":[["copper-plate",1]],"name":"copper-cable","result":"copper-cable","result_count":2,"type":"recipe"}"#;

        #[test]
        fn parses() {
            let expect = Recipe {
                r#type: MustBe!("recipe"),
                name: "copper-cable".into(),
                category: None,
                subgroup: None,
                order: None,
                recipe_data: RecipeDataEnum::Simple(RecipeData {
                    enabled: true,
                    ingredients: vec![Ingredient::SimpleItem("copper-plate".into(), 1)],
                    output: Output::Single(SingleOutput {
                        name: "copper-cable".into(),
                        amount: 2,
                        probability: 1.0,
                    }),
                    duration: Duration::milliseconds(500),
                }),
                icon: None,
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

        const RECIPE: &str = r#"{"category":"centrifuging","enabled":false,"energy_required":12.0,"icon":"__base__/graphics/icons/uranium-processing.png","icon_mipmaps":4,"icon_size":64,"ingredients":[["uranium-ore",10]],"name":"uranium-processing","order":"k[uranium-processing]","results":[{"amount":1,"name":"uranium-235","probability":0.007000000000000001},{"amount":1,"name":"uranium-238","probability":0.993}],"subgroup":"raw-material","type":"recipe"}"#;

        #[test]
        fn parses() {
            let expect = Recipe {
                r#type: MustBe!("recipe"),
                name: "uranium-processing".into(),
                category: Some("centrifuging".into()),
                subgroup: Some("raw-material".into()),
                order: Some("k[uranium-processing]".into()),
                icon: Some(IconData {
                    path: "__base__/graphics/icons/uranium-processing.png".into(),
                    mipmaps: 4,
                    size: 64,
                }),
                recipe_data: RecipeDataEnum::Simple(RecipeData {
                    enabled: false,
                    duration: Duration::seconds(12),
                    ingredients: vec![Ingredient::SimpleItem("uranium-ore".into(), 10)],
                    output: Output::Many(ManyOutputs {
                        outputs: vec![
                            OutputItem {
                                name: "uranium-235".into(),
                                amount: 1,
                                probability: 0.007000000000000001,
                                r#type: None,
                            },
                            OutputItem {
                                name: "uranium-238".into(),
                                amount: 1,
                                probability: 0.993,
                                r#type: None,
                            },
                        ],
                    }),
                }),
            };

            let found = serde_json::from_str::<Recipe>(RECIPE).unwrap();
            assert_eq!(found, expect);
        }

        #[test]
        fn roundtrip() {
            super::roundtrip(RECIPE);
        }
    }

    mod advanced_oil_processing {
        use super::super::*;

        const RECIPE: &str = r#"{"category":"oil-processing","enabled":false,"energy_required":5.0,"icon":"__base__/graphics/icons/fluid/advanced-oil-processing.png","icon_mipmaps":4,"icon_size":64,"ingredients":[{"amount":50,"name":"water","type":"fluid"},{"amount":100,"name":"crude-oil","type":"fluid"}],"name":"advanced-oil-processing","order":"a[oil-processing]-b[advanced-oil-processing]","results":[{"amount":25,"name":"heavy-oil","type":"fluid"},{"amount":45,"name":"light-oil","type":"fluid"},{"amount":55,"name":"petroleum-gas","type":"fluid"}],"subgroup":"fluid-recipes","type":"recipe"}"#;

        #[test]
        fn parses() {
            let expect = Recipe {
                r#type: MustBe!("recipe"),
                name: "advanced-oil-processing".into(),
                category: Some("oil-processing".into()),
                subgroup: Some("fluid-recipes".into()),
                order: Some("a[oil-processing]-b[advanced-oil-processing]".into()),
                icon: Some(IconData {
                    path: "__base__/graphics/icons/fluid/advanced-oil-processing.png".into(),
                    mipmaps: 4,
                    size: 64,
                }),
                recipe_data: RecipeDataEnum::Simple(RecipeData {
                    enabled: false,
                    duration: Duration::seconds(5),
                    ingredients: vec![
                        Ingredient::Fluid(Fluid {
                            amount: 50,
                            name: "water".into(),
                            r#type: MustBe!("fluid"),
                        }),
                        Ingredient::Fluid(Fluid {
                            amount: 100,
                            name: "crude-oil".into(),
                            r#type: MustBe!("fluid"),
                        }),
                    ],
                    output: Output::Many(ManyOutputs {
                        outputs: vec![
                            OutputItem {
                                name: "heavy-oil".into(),
                                amount: 25,
                                probability: 1.0,
                                r#type: Some("fluid".into()),
                            },
                            OutputItem {
                                name: "light-oil".into(),
                                amount: 45,
                                probability: 1.0,
                                r#type: Some("fluid".into()),
                            },
                            OutputItem {
                                name: "petroleum-gas".into(),
                                amount: 55,
                                probability: 1.0,
                                r#type: Some("fluid".into()),
                            },
                        ],
                    }),
                }),
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
