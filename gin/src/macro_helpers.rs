use juniper::{
    DefaultScalarValue, Executor, LookAheadMethods, LookAheadSelection, LookAheadValue, ScalarValue,
};

pub fn int_argument_from_look_ahead(
    look_ahead: &LookAheadSelection<DefaultScalarValue>,
    argument: &str,
    default: i32,
) -> i32 {
    look_ahead
        .argument(argument)
        .map(|arg| {
            if let LookAheadValue::Scalar(first) = arg.value() {
                first.as_int().unwrap_or(default)
            } else {
                default
            }
        })
        .unwrap_or(default)
}
