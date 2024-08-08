use crate::lexer::{Lexer, LiteralTokenType, TokenType};

impl<'a> Lexer<'a> {
    
    pub(crate) fn find_string_or_char_token(&mut self) -> Option<TokenType> {

        if !self.cursor.select_if_starts_with("\"") && !self.cursor.select_if_starts_with("\'"){
            return None;
        }

        let mut rust_lit = String::new(); // The literal as rust literal

        let quote = self.cursor.last_selected();

        let is_char = quote == '\'';

        let check_is_kufr_or_unsupported_char = |c: char|{
            if is_kufr_or_unsupported_character(c) {
                panic!(); // TODO
            }
        };

        let mut closed = false;

        self.cursor.select_while(|remaining: &str, remaining_start: usize|{
            
            let mut chars=remaining.chars();

            let c = chars.next().unwrap_or_default();

            if closed {
                return 0; // Stop selecting any characters
            }

            if c == quote {
                closed = true;
                return 1; // Select the closing quote
            }

            // A normal char
            if c != '\\'{
                check_is_kufr_or_unsupported_char(c);
                rust_lit.push(c);
                return 1; // Select this only char
            }

            let c = chars.next().unwrap_or_default(); // The char after the slash

            // A unicode char
            if c == 'ي' {

                for _ in 0..4 {
                    if !chars.next().is_some_and(|c|{c.is_ascii_hexdigit()}){
                        panic!(); // TODO
                    }
                }

                let code_point=u32::from_str_radix(&remaining[2..6],16).unwrap();

                match char::from_u32(code_point){
                    Some(c) => {
                        check_is_kufr_or_unsupported_char(c);
                        rust_lit.push(c);
                    }
                    None => panic!(), // TODO
                }

                return 6; // Select the `\` and `ي` and the 4 digits of the code-point
            }

            // An escape sequence
            match to_escape_sequence(c) {
                Some(c) => {
                    rust_lit.push(c);
                    return 2; // Select the `\` and the escape char
                },
                None => panic!() // TODO,
            }

            return 0;
        });

        if !closed {
            panic!() // TODO
        }

        if is_char && rust_lit.len() != 1{
            panic!() // TODO
        }

        Some(
            TokenType::Literal(
                if is_char {
                    LiteralTokenType::Char(rust_lit.chars().next().unwrap())
                }
                else {
                    LiteralTokenType::String(rust_lit)
                }
            )
        )
    }
}

fn is_kufr_or_unsupported_character(c:char) -> bool{
    let chars=[
        '\u{03EE}','\u{03EF}','\u{058d}','\u{058e}',
        '\u{05EF}', // yod triangle
        '\u{07D9}','\u{093B}','\u{13D0}','\u{16BE}','\u{165C}','\u{16ED}',
        '\u{17D2}','\u{1D7B}','\u{2020}','\u{2021}','\u{256A}','\u{256B}',
        '\u{256C}','\u{2616}','\u{2617}','\u{269C}','\u{269E}','\u{269F}',
        '\u{26AF}','\u{26B0}','\u{26B1}','\u{26F3}','\u{26F9}','\u{26FB}',
        '\u{26FF}','\u{27CA}','\u{29FE}','\u{2CFE}',
    ];

    if chars.contains(&c){
        return true
    }

    let ranges=[
        /*  from  ,    to  */
        ('\u{0900}','\u{109F}'),//HinduEurope
        ('\u{1100}','\u{1C7F}'),//HinduEurope
        ('\u{253C}','\u{254B}'),
        ('\u{2624}','\u{2638}'),//Kufr
        ('\u{263D}','\u{2653}'),//Kufr
        ('\u{2654}','\u{2667}'),
        ('\u{2669}','\u{2671}'),//Music and kufr crosses
        ('\u{2680}','\u{268F}'),
        ('\u{2680}','\u{268F}'),
        ('\u{26A2}','\u{26A9}'),// Pride
        ('\u{26B3}','\u{26BC}'),// Kufr
        ('\u{26BF}','\u{26EC}'),
        ('\u{2719}','\u{2725}'),// Kufr crosses
        ('\u{2BF0}','\u{2C5F}'),// Includes astrology
        ('\u{2D80}','\u{AB2F}'),
        ('\u{AB70}','\u{FAFF}'),
    ];

    for (r1,r2) in ranges{
        if c>=r1 && c<=r2{
            return true
        }
    }

    return false
}

fn to_escape_sequence(c:char) -> Option<char>{
    match c {
        'خ' => Some('\x08'),   // مسافة للخلف
        'ر' => Some('\x0b'),   // مسافة رأسية
        'ص' => Some('\x0c'),   // الصفحة التالية
        'ف' => Some('\t')  ,   // مسافة أفقية
        'س' => Some('\n')  ,   // سطر جديد
        'ج' => Some('\r')  ,   // إرجاع المؤشر إلى بداية السطر، وبدء الكتابة منه
        '\\'=> Some('\\')  ,
        '\''=> Some('\'')  ,
        '\"'=> Some('\"')  ,
        _   => None
    }
}