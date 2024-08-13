use error::*;

use crate::lexer::*;

impl<'a> Lexer<'a> {
    
    pub(crate) fn find_number_token(&mut self) -> Option<TokenType> {
        if !self.cursor.get_remainder().starts_with(|c: char|{ c.is_ascii_digit() }){
            return None;
        }
        
        match self.cursor.select_next(){
            '0' => todo!(),
            _ => todo!()
        }

        None
    }

}


impl<'a> LexerIter<'a> {

    pub(crate) fn next_num_token(&mut self) -> TokenType {
        todo!()
    }
}