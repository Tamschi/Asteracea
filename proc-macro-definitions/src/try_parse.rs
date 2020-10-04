use syn::{
    parse::{ParseStream, Result},
    MethodTurbofish, Token,
};

pub trait TryParse<T> {
    fn try_parse(self) -> Result<Option<T>>;
}

impl TryParse<MethodTurbofish> for ParseStream<'_> {
    fn try_parse(self) -> Result<Option<MethodTurbofish>> {
        #[allow(unreachable_code)] //TODO
        Ok(if self.peek(Token![::]) && self.peek2(Token![<]) {
            Some(MethodTurbofish {
                colon2_token: self.parse()?,
                lt_token: self.parse()?,
                args: todo!("TryParse<MethodTurbofish>"),
                gt_token: self.parse()?,
            })
        } else {
            None
        })
    }
}
