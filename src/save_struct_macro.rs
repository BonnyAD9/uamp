#[macro_export]
macro_rules! gen_struct {
    (
        $(#$sat:tt)*
        $sv:vis $t:ident;
        $(
            $(#$at:tt)*
            $fv:vis $fi:ident: $ft:ty {
                $gfv:vis $gffi:ident, $sfv:vis $sffi:ident
            } $(=> $defv:vis $defi:ident $defl:literal: $def:expr)?
        ),* $(,)?
        ;$(
            $(#$dat:tt)*
            $dfv:vis $dfi:ident: $dft:ty {
                $dgfv:vis $dgffi:ident, $dsfv:vis $dsffi:ident
            } $(=> $ddefv:vis $ddefi:ident $ddefl:literal: $ddef:expr)?
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
                $gfv fn $gffi(&self) -> &$ft {
                    &self.$fi
                }

                $sfv fn $sffi(&mut self) -> &mut $ft {
                    self.change.set(true);
                    &mut self.$fi
                }
            )*
            $(
                $dgfv fn $dgffi(&self) -> $dft {
                    self.$dfi
                }

                $dsfv fn $dsffi(&mut self, v: $dft) {
                    if self.$dfi != v {
                        self.change.set(true);
                        self.$dfi = v;
                    }
                }
            )*
        }

        $($(
            $defv fn $defi() -> $ft {
                $def
            }
        )?)*
        $($(
            $ddefv fn $ddefi() -> $dft {
                $ddef
            }
        )?)*
    };
}
