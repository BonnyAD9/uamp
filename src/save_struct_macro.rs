#[macro_export]
macro_rules! gen_struct {
    (
        $(#$sat:tt)*
        $sv:vis $t:ident {
            $(
                $(#$at:tt)*
                $fv:vis $fi:ident: $ft:ty { $gfv:vis, $sfv:vis }
                    $($defv:vis fn $defl:literal: $def:expr)?
            ),* $(,)?
            ;$(
                $(#$dat:tt)*
                $dfv:vis $dfi:ident: $dft:ty { $dgfv:vis, $dsfv:vis }
                    $($ddefv:vis fn $ddefl:literal: $ddef:expr)?
            ),* $(,)?
            ;$(
                $(#$rat:tt)*
                $rfv:vis $rfi:ident: $rft:ty
            ),* $(,)?
        }
    ) => {
        #[derive(Serialize, Deserialize)]
        $(#$sat)*
        $sv struct $t {
            $(
                $(#$at)*
                $(#[serde(default = $defl)])?
                $fv $fi: $ft,
            )*
            $(
                $(#$dat)*
                $(#[serde(default = $ddefl)])?
                $dfv $dfi: $dft,
            )*
            $(
                $(#$rat)*
                $rfv $rfi: $rft,
            )*
            #[serde(skip)]
            change: std::cell::Cell<bool>,
        }

        impl $t {
            $(
                $gfv fn $fi(&self) -> &$ft {
                    &self.$fi
                }

                paste::paste! {
                    $sfv fn [<$fi _mut>](&mut self) -> &mut $ft {
                        self.change.set(true);
                        &mut self.$fi
                    }
                }

            )*
            $(
                $dgfv fn $dfi(&self) -> $dft {
                    self.$dfi
                }

                paste::paste! {
                    $dsfv fn [<$dfi _set>](&mut self, v: $dft) {
                        if self.$dfi != v {
                            self.change.set(true);
                            self.$dfi = v;
                        }
                    }
                }
            )*
        }

        $($(paste::paste! {
            $defv fn [<default_ $fi>]() -> $ft {
                $def
            }
        })?)*
        $($(paste::paste! {
            $ddefv fn [<default_ $dfi>]() -> $dft {
                $ddef
            }
        })?)*
    };
}
