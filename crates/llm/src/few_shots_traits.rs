use serde::{Deserialize, Serialize};

pub trait FewShotsOutput<T> {
    fn few_shots() -> Vec<T>;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InputOutputFewShot<T> {
    input: String,
    output: T,
}

pub trait FewShotsOutputs<T> {
    fn few_shots_outputs() -> Vec<InputOutputFewShot<T>>;
}
