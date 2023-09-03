/// Generates struct with setters and getters.
///
/// The default values are used with serde.
///
/// # Examples:
/// ```
/// gen_struct!{
///     #[attributes]
///     pub StructName {
///         // Fields passed by reference
///         #[attributes]
///         // ...
///         ref_field: FieldType {
///             /* getter visibility */ pub,
///             /* setter visibility */ pub
///         } => /* default function visibility */ pub default_value_expr(),
///         // ...
///         ; // Fields passed by value
///         #[attributes]
///         // ...
///         value_field: FieldType { pub, pub } => pub () default_value_expr(),
///         // ...
///         ; // Other fields that are not parsed by serde
///         #[serde(ignore)] // the serde ignore is optional, but it should be
///                          // there
///         #[attributes]
///         // ...
///         field3: FieldType,
///         // ...
///     }
/// }
/// ```
/// The part after the setters (`=> pub () ...`) is optional.
///
/// All the visibility modifiers can be omited to make it private.
/// The setters and getters are generated in the following shape:
/// ```
/// pub fn ref_field(&self) -> &FieldType { /* ... */ }
/// pub fn ref_field_mut(&mut self) -> &mut FieldType { /* ... */ }
/// pub fn value_field(&self) -> FieldType { /* ... */ }
/// pub fn value_field_set(&mut self, v: FieldType) { /* ... */ }
/// ```
///
/// The default value functions are generated as follows:
/// ```
/// pub fn default_ref_field() -> FieldType { /* ... */ }
/// pub fn default_value_field() -> FieldType { /* ... */ }
/// ```
///
/// The macro also adds a field to the structure:
/// ```
/// change: std::cell::Cell<bool>;
/// ```
/// This is set to true every time a setter function is called.
#[macro_export]
macro_rules! gen_struct {
    (
        $(#$sat:tt)*
        $sv:vis $t:ident {
            $(
                $(#$at:tt)*
                $fv:vis $fi:ident: $ft:ty { $gfv:vis $(pri)?, $sfv:vis $(pri)? }
                    $( => $defv:vis $(pri)? ($($n:literal)?) $def:expr)?,
            )*
            ;$(
                $(#$dat:tt)*
                $dfv:vis $dfi:ident: $dft:ty { $dgfv:vis $(pri)?, $dsfv:vis $(pri)? }
                    $( => $ddefv:vis $(pri)? ($($dn:literal)?) $ddef:expr )?,
            )*
            ;$(
                $(#$rat:tt)*
                $rfv:vis $rfi:ident: $rft:ty,
            )*
            $(;$(#$skipat:tt)*)?
        }
    ) => {
        place_macro::place!{
            $(#$sat)*
            $sv struct $t {
                $(
                    $(#$at)*
                    $(#[serde(default = __string__("default_" $fi $($n)?))])?
                    $fv $fi: $ft,
                )*
                $(
                    $(#$dat)*
                    $(#[serde(default = __string__("default_" $dfi $($dn)?))])?
                    $dfv $dfi: $dft,
                )*
                $(
                    $(#$rat)*
                    $rfv $rfi: $rft,
                )*
                $($(#$skipat)*)?
                change: std::cell::Cell<bool>,
            }
        }

        impl $t {
            $(
                $gfv fn $fi(&self) -> &$ft {
                    &self.$fi
                }

                place_macro::place! {
                    $sfv fn __ident__($fi _mut)(&mut self) -> &mut $ft {
                        self.change.set(true);
                        &mut self.$fi
                    }
                }

            )*
            $(
                $dgfv fn $dfi(&self) -> $dft {
                    self.$dfi
                }

                place_macro::place! {
                    $dsfv fn __ident__($dfi _set)(&mut self, v: $dft) {
                        if self.$dfi != v {
                            self.change.set(true);
                            self.$dfi = v;
                        }
                    }
                }
            )*
        }

        $($(place_macro::place! {
            $defv fn __ident__(default_ $fi)() -> $ft {
                $def
            }
        })?)*
        $($(place_macro::place! {
            $ddefv fn __ident__(default_ $dfi)() -> $dft {
                $ddef
            }
        })?)*
    };
}
