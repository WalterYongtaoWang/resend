#[macro_export]
#[allow(clippy::crate_in_macro_def)]
macro_rules! snd_ref {
    ($t:ty) => {
        impl Sendable for $t {
            #[inline]
            fn snd_to<W>(&self, writer: &mut W) -> crate::Result<()>
            where
                W: Sender,
            {
                (*self).snd_to(writer)
            }
        }
    };
}

#[macro_export]
#[allow(clippy::crate_in_macro_def)]
macro_rules! impl_tuple {
    ($($name:ident), +) => {
        impl<$($name: Sendable),+> Sendable for ($($name,)+)
        {
            #[allow(non_snake_case)]
            #[inline]
            fn snd_to<W>(&self, writer: &mut W) -> crate::Result<()> 
            where W: Sender
            {
                let ($($name,)+) = self;
                $($name.snd_to(writer)?;)+
                Ok(())
            }
        }

        impl<$($name: Receivable),+> Receivable for ($($name,)+)
        {
            #[inline]
            fn rcv_from<R>(reader: &mut R) -> crate::Result<Self>
            where
                R: Receiver 
            {
                Ok(
                    (
                        $($name::rcv_from(reader)?,)+
                    )
                )
            }
        }

    };
}
