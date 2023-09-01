#[macro_export]
macro_rules! gen_struct {
    (
        $(#$sat:tt)*
        $sv:vis $t:ident;
        $(
            $(#$at:tt)*
            $fv:vis $fi:ident: $ft:ty { $gfv:vis, $sfv:vis }
                $($defv:vis is $defl:literal: $def:expr)?
        ),* $(,)?
        ;$(
            $(#$dat:tt)*
            $dfv:vis $dfi:ident: $dft:ty { $dgfv:vis, $dsfv:vis }
                $($ddefv:vis is $ddefl:literal: $ddef:expr)?
        ),* $(,)?
        ;$(
            $(#$rat:tt)*
            $rfv:vis $rfi:ident: $rft:ty
        ),* $(,)?
    ) => {
        $(#$sat)*
        #[derive(Serialize, Deserialize)]
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

                paste::item! {
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

                paste::item! {
                    $dsfv fn [<$dfi _set>](&mut self, v: $dft) {
                        if self.$dfi != v {
                            self.change.set(true);
                            self.$dfi = v;
                        }
                    }
                }
            )*
        }

        $($(paste::item! {
            $defv fn [<default_ $fi>]() -> $ft {
                $def
            }
        })?)*
        $($(paste::item! {
            $ddefv fn [<default_ $dfi>]() -> $dft {
                $ddef
            }
        })?)*
    };
}
