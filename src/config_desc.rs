/// Represents types able to describe a set of [configuration values][1].
///
/// [1]: crate::readers
pub trait ConfigDescriptor: Sized {
    /// Creates the configuration descriptor.
    fn define() -> Self;
}
