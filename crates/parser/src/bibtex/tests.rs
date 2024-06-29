use expect_test::{expect, Expect};

use crate::parse_bibtex;

fn check(input: &str, expect: Expect) {
    let root = syntax::bibtex::SyntaxNode::new_root(parse_bibtex(input));
    expect.assert_debug_eq(&root);
}

#[test]
fn test_smoke() {
    check(
        r#""#,
        expect![[r#"
        ROOT@0..0

    "#]],
    );
}

#[test]
fn test_preamble() {
    check(
        r#"@article{test, author = {\"{\i}}}"#,
        expect![[r#"
        ROOT@0..33
          ENTRY@0..33
            TYPE@0..8 "@article"
            L_DELIM@8..9 "{"
            NAME@9..13 "test"
            COMMA@13..14 ","
            WHITESPACE@14..15 " "
            FIELD@15..32
              NAME@15..21 "author"
              WHITESPACE@21..22 " "
              EQ@22..23 "="
              WHITESPACE@23..24 " "
              CURLY_GROUP@24..32
                L_CURLY@24..25 "{"
                ACCENT@25..31
                  ACCENT_NAME@25..27 "\\\""
                  L_CURLY@27..28 "{"
                  COMMAND_NAME@28..30 "\\i"
                  R_CURLY@30..31 "}"
                R_CURLY@31..32 "}"
            R_DELIM@32..33 "}"

    "#]],
    );
}

#[test]
fn test_comment() {
    check(
        r#"Some junk here

@comment{foo,
    author = {Foo Bar},
    title = {Some Title},
}
"#,
        expect![[r#"
            ROOT@0..82
              JUNK@0..16 "Some junk here\n\n"
              TYPE@16..24 "@comment"
              JUNK@24..82 "{foo,\n    author = {F ..."

        "#]],
    );
}

#[test]
fn test_issue_809() {
    check(
        r#"@article{issue_809,
  title = {foo (" bar) baz},
}"#,
        expect![[r#"
            ROOT@0..50
              ENTRY@0..50
                TYPE@0..8 "@article"
                L_DELIM@8..9 "{"
                NAME@9..18 "issue_809"
                COMMA@18..19 ","
                WHITESPACE@19..22 "\n  "
                FIELD@22..48
                  NAME@22..27 "title"
                  WHITESPACE@27..28 " "
                  EQ@28..29 "="
                  WHITESPACE@29..30 " "
                  CURLY_GROUP@30..47
                    L_CURLY@30..31 "{"
                    WORD@31..34 "foo"
                    WHITESPACE@34..35 " "
                    WORD@35..36 "("
                    QUOTE@36..37 "\""
                    WHITESPACE@37..38 " "
                    WORD@38..42 "bar)"
                    WHITESPACE@42..43 " "
                    WORD@43..46 "baz"
                    R_CURLY@46..47 "}"
                  COMMA@47..48 ","
                WHITESPACE@48..49 "\n"
                R_DELIM@49..50 "}"

        "#]],
    );
}

#[test]
fn test_issue_945() {
    check(
        r#"@article{test, author = {\"{\i}}}"#,
        expect![[r#"
        ROOT@0..33
          ENTRY@0..33
            TYPE@0..8 "@article"
            L_DELIM@8..9 "{"
            NAME@9..13 "test"
            COMMA@13..14 ","
            WHITESPACE@14..15 " "
            FIELD@15..32
              NAME@15..21 "author"
              WHITESPACE@21..22 " "
              EQ@22..23 "="
              WHITESPACE@23..24 " "
              CURLY_GROUP@24..32
                L_CURLY@24..25 "{"
                ACCENT@25..31
                  ACCENT_NAME@25..27 "\\\""
                  L_CURLY@27..28 "{"
                  COMMAND_NAME@28..30 "\\i"
                  R_CURLY@30..31 "}"
                R_CURLY@31..32 "}"
            R_DELIM@32..33 "}"

    "#]],
    );
}

#[test]
fn test_aho_2006() {
    check(
        r#"@book{10.5555/1177220,
    author = {Aho, Alfred V. and Lam, Monica S. and Sethi, Ravi and Ullman, Jeffrey D.},
    title = {Compilers: Principles, Techniques, and Tools (2nd Edition)},
    year = {2006},
    isbn = {0321486811},
    publisher = {Addison-Wesley Longman Publishing Co., Inc.},
    address = {USA}
}"#,
        expect![[r#"
            ROOT@0..314
              ENTRY@0..314
                TYPE@0..5 "@book"
                L_DELIM@5..6 "{"
                NAME@6..21 "10.5555/1177220"
                COMMA@21..22 ","
                WHITESPACE@22..27 "\n    "
                FIELD@27..111
                  NAME@27..33 "author"
                  WHITESPACE@33..34 " "
                  EQ@34..35 "="
                  WHITESPACE@35..36 " "
                  CURLY_GROUP@36..110
                    L_CURLY@36..37 "{"
                    WORD@37..40 "Aho"
                    COMMA@40..41 ","
                    WHITESPACE@41..42 " "
                    WORD@42..48 "Alfred"
                    WHITESPACE@48..49 " "
                    WORD@49..51 "V."
                    WHITESPACE@51..52 " "
                    WORD@52..55 "and"
                    WHITESPACE@55..56 " "
                    WORD@56..59 "Lam"
                    COMMA@59..60 ","
                    WHITESPACE@60..61 " "
                    WORD@61..67 "Monica"
                    WHITESPACE@67..68 " "
                    WORD@68..70 "S."
                    WHITESPACE@70..71 " "
                    WORD@71..74 "and"
                    WHITESPACE@74..75 " "
                    WORD@75..80 "Sethi"
                    COMMA@80..81 ","
                    WHITESPACE@81..82 " "
                    WORD@82..86 "Ravi"
                    WHITESPACE@86..87 " "
                    WORD@87..90 "and"
                    WHITESPACE@90..91 " "
                    WORD@91..97 "Ullman"
                    COMMA@97..98 ","
                    WHITESPACE@98..99 " "
                    WORD@99..106 "Jeffrey"
                    WHITESPACE@106..107 " "
                    WORD@107..109 "D."
                    R_CURLY@109..110 "}"
                  COMMA@110..111 ","
                WHITESPACE@111..116 "\n    "
                FIELD@116..185
                  NAME@116..121 "title"
                  WHITESPACE@121..122 " "
                  EQ@122..123 "="
                  WHITESPACE@123..124 " "
                  CURLY_GROUP@124..184
                    L_CURLY@124..125 "{"
                    WORD@125..135 "Compilers:"
                    WHITESPACE@135..136 " "
                    WORD@136..146 "Principles"
                    COMMA@146..147 ","
                    WHITESPACE@147..148 " "
                    WORD@148..158 "Techniques"
                    COMMA@158..159 ","
                    WHITESPACE@159..160 " "
                    WORD@160..163 "and"
                    WHITESPACE@163..164 " "
                    WORD@164..169 "Tools"
                    WHITESPACE@169..170 " "
                    WORD@170..174 "(2nd"
                    WHITESPACE@174..175 " "
                    WORD@175..183 "Edition)"
                    R_CURLY@183..184 "}"
                  COMMA@184..185 ","
                WHITESPACE@185..190 "\n    "
                FIELD@190..204
                  NAME@190..194 "year"
                  WHITESPACE@194..195 " "
                  EQ@195..196 "="
                  WHITESPACE@196..197 " "
                  CURLY_GROUP@197..203
                    L_CURLY@197..198 "{"
                    INTEGER@198..202 "2006"
                    R_CURLY@202..203 "}"
                  COMMA@203..204 ","
                WHITESPACE@204..209 "\n    "
                FIELD@209..229
                  NAME@209..213 "isbn"
                  WHITESPACE@213..214 " "
                  EQ@214..215 "="
                  WHITESPACE@215..216 " "
                  CURLY_GROUP@216..228
                    L_CURLY@216..217 "{"
                    INTEGER@217..227 "0321486811"
                    R_CURLY@227..228 "}"
                  COMMA@228..229 ","
                WHITESPACE@229..234 "\n    "
                FIELD@234..292
                  NAME@234..243 "publisher"
                  WHITESPACE@243..244 " "
                  EQ@244..245 "="
                  WHITESPACE@245..246 " "
                  CURLY_GROUP@246..291
                    L_CURLY@246..247 "{"
                    WORD@247..261 "Addison-Wesley"
                    WHITESPACE@261..262 " "
                    WORD@262..269 "Longman"
                    WHITESPACE@269..270 " "
                    WORD@270..280 "Publishing"
                    WHITESPACE@280..281 " "
                    WORD@281..284 "Co."
                    COMMA@284..285 ","
                    WHITESPACE@285..286 " "
                    WORD@286..290 "Inc."
                    R_CURLY@290..291 "}"
                  COMMA@291..292 ","
                WHITESPACE@292..297 "\n    "
                FIELD@297..313
                  NAME@297..304 "address"
                  WHITESPACE@304..305 " "
                  EQ@305..306 "="
                  WHITESPACE@306..307 " "
                  CURLY_GROUP@307..312
                    L_CURLY@307..308 "{"
                    WORD@308..311 "USA"
                    R_CURLY@311..312 "}"
                  WHITESPACE@312..313 "\n"
                R_DELIM@313..314 "}"

        "#]],
    );
}

#[test]
fn test_aksin_2006() {
    check(
        r#"@string{jomch   = {J.~Organomet. Chem.}}

@article{aksin,
    author       = {Aks{\i}n, {\"O}zge and T{\"u}rkmen, Hayati and Artok, Levent
                    and {\c{C}}etinkaya, Bekir and Ni, Chaoying and
                    B{\"u}y{\"u}kg{\"u}ng{\"o}r, Orhan and {\"O}zkal, Erhan},
    title        = {Effect of immobilization on catalytic characteristics of
                    saturated {Pd-N}-heterocyclic carbenes in {Mizoroki-Heck}
                    reactions},
    journaltitle = jomch,
    date         = 2006,
    volume       = 691,
    number       = 13,
    pages        = {3027-3036},
    indextitle   = {Effect of immobilization on catalytic characteristics},
}"#,
        expect![[r#"
            ROOT@0..679
              STRING@0..40
                TYPE@0..7 "@string"
                L_DELIM@7..8 "{"
                NAME@8..13 "jomch"
                WHITESPACE@13..16 "   "
                EQ@16..17 "="
                WHITESPACE@17..18 " "
                CURLY_GROUP@18..39
                  L_CURLY@18..19 "{"
                  WORD@19..21 "J."
                  NBSP@21..22 "~"
                  WORD@22..32 "Organomet."
                  WHITESPACE@32..33 " "
                  WORD@33..38 "Chem."
                  R_CURLY@38..39 "}"
                R_DELIM@39..40 "}"
              JUNK@40..42 "\n\n"
              ENTRY@42..679
                TYPE@42..50 "@article"
                L_DELIM@50..51 "{"
                NAME@51..56 "aksin"
                COMMA@56..57 ","
                WHITESPACE@57..62 "\n    "
                FIELD@62..284
                  NAME@62..68 "author"
                  WHITESPACE@68..75 "       "
                  EQ@75..76 "="
                  WHITESPACE@76..77 " "
                  CURLY_GROUP@77..283
                    L_CURLY@77..78 "{"
                    WORD@78..81 "Aks"
                    CURLY_GROUP@81..85
                      L_CURLY@81..82 "{"
                      COMMAND@82..84
                        COMMAND_NAME@82..84 "\\i"
                      R_CURLY@84..85 "}"
                    WORD@85..86 "n"
                    COMMA@86..87 ","
                    WHITESPACE@87..88 " "
                    CURLY_GROUP@88..93
                      L_CURLY@88..89 "{"
                      ACCENT@89..92
                        ACCENT_NAME@89..91 "\\\""
                        WORD@91..92 "O"
                      R_CURLY@92..93 "}"
                    WORD@93..96 "zge"
                    WHITESPACE@96..97 " "
                    WORD@97..100 "and"
                    WHITESPACE@100..101 " "
                    WORD@101..102 "T"
                    CURLY_GROUP@102..107
                      L_CURLY@102..103 "{"
                      ACCENT@103..106
                        ACCENT_NAME@103..105 "\\\""
                        WORD@105..106 "u"
                      R_CURLY@106..107 "}"
                    WORD@107..112 "rkmen"
                    COMMA@112..113 ","
                    WHITESPACE@113..114 " "
                    WORD@114..120 "Hayati"
                    WHITESPACE@120..121 " "
                    WORD@121..124 "and"
                    WHITESPACE@124..125 " "
                    WORD@125..130 "Artok"
                    COMMA@130..131 ","
                    WHITESPACE@131..132 " "
                    WORD@132..138 "Levent"
                    WHITESPACE@138..159 "\n                    "
                    WORD@159..162 "and"
                    WHITESPACE@162..163 " "
                    CURLY_GROUP@163..170
                      L_CURLY@163..164 "{"
                      ACCENT@164..169
                        ACCENT_NAME@164..166 "\\c"
                        L_CURLY@166..167 "{"
                        WORD@167..168 "C"
                        R_CURLY@168..169 "}"
                      R_CURLY@169..170 "}"
                    WORD@170..178 "etinkaya"
                    COMMA@178..179 ","
                    WHITESPACE@179..180 " "
                    WORD@180..185 "Bekir"
                    WHITESPACE@185..186 " "
                    WORD@186..189 "and"
                    WHITESPACE@189..190 " "
                    WORD@190..192 "Ni"
                    COMMA@192..193 ","
                    WHITESPACE@193..194 " "
                    WORD@194..202 "Chaoying"
                    WHITESPACE@202..203 " "
                    WORD@203..206 "and"
                    WHITESPACE@206..227 "\n                    "
                    WORD@227..228 "B"
                    CURLY_GROUP@228..233
                      L_CURLY@228..229 "{"
                      ACCENT@229..232
                        ACCENT_NAME@229..231 "\\\""
                        WORD@231..232 "u"
                      R_CURLY@232..233 "}"
                    WORD@233..234 "y"
                    CURLY_GROUP@234..239
                      L_CURLY@234..235 "{"
                      ACCENT@235..238
                        ACCENT_NAME@235..237 "\\\""
                        WORD@237..238 "u"
                      R_CURLY@238..239 "}"
                    WORD@239..241 "kg"
                    CURLY_GROUP@241..246
                      L_CURLY@241..242 "{"
                      ACCENT@242..245
                        ACCENT_NAME@242..244 "\\\""
                        WORD@244..245 "u"
                      R_CURLY@245..246 "}"
                    WORD@246..248 "ng"
                    CURLY_GROUP@248..253
                      L_CURLY@248..249 "{"
                      ACCENT@249..252
                        ACCENT_NAME@249..251 "\\\""
                        WORD@251..252 "o"
                      R_CURLY@252..253 "}"
                    WORD@253..254 "r"
                    COMMA@254..255 ","
                    WHITESPACE@255..256 " "
                    WORD@256..261 "Orhan"
                    WHITESPACE@261..262 " "
                    WORD@262..265 "and"
                    WHITESPACE@265..266 " "
                    CURLY_GROUP@266..271
                      L_CURLY@266..267 "{"
                      ACCENT@267..270
                        ACCENT_NAME@267..269 "\\\""
                        WORD@269..270 "O"
                      R_CURLY@270..271 "}"
                    WORD@271..275 "zkal"
                    COMMA@275..276 ","
                    WHITESPACE@276..277 " "
                    WORD@277..282 "Erhan"
                    R_CURLY@282..283 "}"
                  COMMA@283..284 ","
                WHITESPACE@284..289 "\n    "
                FIELD@289..471
                  NAME@289..294 "title"
                  WHITESPACE@294..302 "        "
                  EQ@302..303 "="
                  WHITESPACE@303..304 " "
                  CURLY_GROUP@304..470
                    L_CURLY@304..305 "{"
                    WORD@305..311 "Effect"
                    WHITESPACE@311..312 " "
                    WORD@312..314 "of"
                    WHITESPACE@314..315 " "
                    WORD@315..329 "immobilization"
                    WHITESPACE@329..330 " "
                    WORD@330..332 "on"
                    WHITESPACE@332..333 " "
                    WORD@333..342 "catalytic"
                    WHITESPACE@342..343 " "
                    WORD@343..358 "characteristics"
                    WHITESPACE@358..359 " "
                    WORD@359..361 "of"
                    WHITESPACE@361..382 "\n                    "
                    WORD@382..391 "saturated"
                    WHITESPACE@391..392 " "
                    CURLY_GROUP@392..398
                      L_CURLY@392..393 "{"
                      WORD@393..397 "Pd-N"
                      R_CURLY@397..398 "}"
                    WORD@398..411 "-heterocyclic"
                    WHITESPACE@411..412 " "
                    WORD@412..420 "carbenes"
                    WHITESPACE@420..421 " "
                    WORD@421..423 "in"
                    WHITESPACE@423..424 " "
                    CURLY_GROUP@424..439
                      L_CURLY@424..425 "{"
                      WORD@425..438 "Mizoroki-Heck"
                      R_CURLY@438..439 "}"
                    WHITESPACE@439..460 "\n                    "
                    WORD@460..469 "reactions"
                    R_CURLY@469..470 "}"
                  COMMA@470..471 ","
                WHITESPACE@471..476 "\n    "
                FIELD@476..497
                  NAME@476..488 "journaltitle"
                  WHITESPACE@488..489 " "
                  EQ@489..490 "="
                  WHITESPACE@490..491 " "
                  LITERAL@491..496
                    NAME@491..496 "jomch"
                  COMMA@496..497 ","
                WHITESPACE@497..502 "\n    "
                FIELD@502..522
                  NAME@502..506 "date"
                  WHITESPACE@506..515 "         "
                  EQ@515..516 "="
                  WHITESPACE@516..517 " "
                  LITERAL@517..521
                    INTEGER@517..521 "2006"
                  COMMA@521..522 ","
                WHITESPACE@522..527 "\n    "
                FIELD@527..546
                  NAME@527..533 "volume"
                  WHITESPACE@533..540 "       "
                  EQ@540..541 "="
                  WHITESPACE@541..542 " "
                  LITERAL@542..545
                    INTEGER@542..545 "691"
                  COMMA@545..546 ","
                WHITESPACE@546..551 "\n    "
                FIELD@551..569
                  NAME@551..557 "number"
                  WHITESPACE@557..564 "       "
                  EQ@564..565 "="
                  WHITESPACE@565..566 " "
                  LITERAL@566..568
                    INTEGER@566..568 "13"
                  COMMA@568..569 ","
                WHITESPACE@569..574 "\n    "
                FIELD@574..601
                  NAME@574..579 "pages"
                  WHITESPACE@579..587 "        "
                  EQ@587..588 "="
                  WHITESPACE@588..589 " "
                  CURLY_GROUP@589..600
                    L_CURLY@589..590 "{"
                    WORD@590..599 "3027-3036"
                    R_CURLY@599..600 "}"
                  COMMA@600..601 ","
                WHITESPACE@601..606 "\n    "
                FIELD@606..677
                  NAME@606..616 "indextitle"
                  WHITESPACE@616..619 "   "
                  EQ@619..620 "="
                  WHITESPACE@620..621 " "
                  CURLY_GROUP@621..676
                    L_CURLY@621..622 "{"
                    WORD@622..628 "Effect"
                    WHITESPACE@628..629 " "
                    WORD@629..631 "of"
                    WHITESPACE@631..632 " "
                    WORD@632..646 "immobilization"
                    WHITESPACE@646..647 " "
                    WORD@647..649 "on"
                    WHITESPACE@649..650 " "
                    WORD@650..659 "catalytic"
                    WHITESPACE@659..660 " "
                    WORD@660..675 "characteristics"
                    R_CURLY@675..676 "}"
                  COMMA@676..677 ","
                WHITESPACE@677..678 "\n"
                R_DELIM@678..679 "}"

        "#]],
    );
}

#[test]
fn test_almendro_1998() {
    check(
        r#"@patent{almendro,
    author       = {Almendro, Jos{\'e} L. and Mart{\'i}n, Jacinto and S{\'a}nchez,
                    Alberto and Nozal, Fernando},
    title        = {Elektromagnetisches Signalhorn},
    number       = {EU-29702195U},
    date         = 1998,
    location     = {countryfr and countryuk and countryde},
    langid       = {german},
    annotation   = {This is a \texttt{patent} entry with a \texttt{location}
                    field. The number is given in the \texttt{number} field. Note
                    the format of the \texttt{location} field in the database
                    file. Compare \texttt{laufenberg}, \texttt{sorace}, and
                    \texttt{kowalik}},
}"#,
        expect![[r#"
            ROOT@0..706
              ENTRY@0..706
                TYPE@0..7 "@patent"
                L_DELIM@7..8 "{"
                NAME@8..16 "almendro"
                COMMA@16..17 ","
                WHITESPACE@17..22 "\n    "
                FIELD@22..150
                  NAME@22..28 "author"
                  WHITESPACE@28..35 "       "
                  EQ@35..36 "="
                  WHITESPACE@36..37 " "
                  CURLY_GROUP@37..149
                    L_CURLY@37..38 "{"
                    WORD@38..46 "Almendro"
                    COMMA@46..47 ","
                    WHITESPACE@47..48 " "
                    WORD@48..51 "Jos"
                    CURLY_GROUP@51..56
                      L_CURLY@51..52 "{"
                      ACCENT@52..55
                        ACCENT_NAME@52..54 "\\'"
                        WORD@54..55 "e"
                      R_CURLY@55..56 "}"
                    WHITESPACE@56..57 " "
                    WORD@57..59 "L."
                    WHITESPACE@59..60 " "
                    WORD@60..63 "and"
                    WHITESPACE@63..64 " "
                    WORD@64..68 "Mart"
                    CURLY_GROUP@68..73
                      L_CURLY@68..69 "{"
                      ACCENT@69..72
                        ACCENT_NAME@69..71 "\\'"
                        WORD@71..72 "i"
                      R_CURLY@72..73 "}"
                    WORD@73..74 "n"
                    COMMA@74..75 ","
                    WHITESPACE@75..76 " "
                    WORD@76..83 "Jacinto"
                    WHITESPACE@83..84 " "
                    WORD@84..87 "and"
                    WHITESPACE@87..88 " "
                    WORD@88..89 "S"
                    CURLY_GROUP@89..94
                      L_CURLY@89..90 "{"
                      ACCENT@90..93
                        ACCENT_NAME@90..92 "\\'"
                        WORD@92..93 "a"
                      R_CURLY@93..94 "}"
                    WORD@94..99 "nchez"
                    COMMA@99..100 ","
                    WHITESPACE@100..121 "\n                    "
                    WORD@121..128 "Alberto"
                    WHITESPACE@128..129 " "
                    WORD@129..132 "and"
                    WHITESPACE@132..133 " "
                    WORD@133..138 "Nozal"
                    COMMA@138..139 ","
                    WHITESPACE@139..140 " "
                    WORD@140..148 "Fernando"
                    R_CURLY@148..149 "}"
                  COMMA@149..150 ","
                WHITESPACE@150..155 "\n    "
                FIELD@155..203
                  NAME@155..160 "title"
                  WHITESPACE@160..168 "        "
                  EQ@168..169 "="
                  WHITESPACE@169..170 " "
                  CURLY_GROUP@170..202
                    L_CURLY@170..171 "{"
                    WORD@171..190 "Elektromagnetisches"
                    WHITESPACE@190..191 " "
                    WORD@191..201 "Signalhorn"
                    R_CURLY@201..202 "}"
                  COMMA@202..203 ","
                WHITESPACE@203..208 "\n    "
                FIELD@208..238
                  NAME@208..214 "number"
                  WHITESPACE@214..221 "       "
                  EQ@221..222 "="
                  WHITESPACE@222..223 " "
                  CURLY_GROUP@223..237
                    L_CURLY@223..224 "{"
                    WORD@224..236 "EU-29702195U"
                    R_CURLY@236..237 "}"
                  COMMA@237..238 ","
                WHITESPACE@238..243 "\n    "
                FIELD@243..263
                  NAME@243..247 "date"
                  WHITESPACE@247..256 "         "
                  EQ@256..257 "="
                  WHITESPACE@257..258 " "
                  LITERAL@258..262
                    INTEGER@258..262 "1998"
                  COMMA@262..263 ","
                WHITESPACE@263..268 "\n    "
                FIELD@268..323
                  NAME@268..276 "location"
                  WHITESPACE@276..281 "     "
                  EQ@281..282 "="
                  WHITESPACE@282..283 " "
                  CURLY_GROUP@283..322
                    L_CURLY@283..284 "{"
                    WORD@284..293 "countryfr"
                    WHITESPACE@293..294 " "
                    WORD@294..297 "and"
                    WHITESPACE@297..298 " "
                    WORD@298..307 "countryuk"
                    WHITESPACE@307..308 " "
                    WORD@308..311 "and"
                    WHITESPACE@311..312 " "
                    WORD@312..321 "countryde"
                    R_CURLY@321..322 "}"
                  COMMA@322..323 ","
                WHITESPACE@323..328 "\n    "
                FIELD@328..352
                  NAME@328..334 "langid"
                  WHITESPACE@334..341 "       "
                  EQ@341..342 "="
                  WHITESPACE@342..343 " "
                  CURLY_GROUP@343..351
                    L_CURLY@343..344 "{"
                    WORD@344..350 "german"
                    R_CURLY@350..351 "}"
                  COMMA@351..352 ","
                WHITESPACE@352..357 "\n    "
                FIELD@357..704
                  NAME@357..367 "annotation"
                  WHITESPACE@367..370 "   "
                  EQ@370..371 "="
                  WHITESPACE@371..372 " "
                  CURLY_GROUP@372..703
                    L_CURLY@372..373 "{"
                    WORD@373..377 "This"
                    WHITESPACE@377..378 " "
                    WORD@378..380 "is"
                    WHITESPACE@380..381 " "
                    WORD@381..382 "a"
                    WHITESPACE@382..383 " "
                    COMMAND@383..390
                      COMMAND_NAME@383..390 "\\texttt"
                    CURLY_GROUP@390..398
                      L_CURLY@390..391 "{"
                      WORD@391..397 "patent"
                      R_CURLY@397..398 "}"
                    WHITESPACE@398..399 " "
                    WORD@399..404 "entry"
                    WHITESPACE@404..405 " "
                    WORD@405..409 "with"
                    WHITESPACE@409..410 " "
                    WORD@410..411 "a"
                    WHITESPACE@411..412 " "
                    COMMAND@412..419
                      COMMAND_NAME@412..419 "\\texttt"
                    CURLY_GROUP@419..429
                      L_CURLY@419..420 "{"
                      WORD@420..428 "location"
                      R_CURLY@428..429 "}"
                    WHITESPACE@429..450 "\n                    "
                    WORD@450..456 "field."
                    WHITESPACE@456..457 " "
                    WORD@457..460 "The"
                    WHITESPACE@460..461 " "
                    WORD@461..467 "number"
                    WHITESPACE@467..468 " "
                    WORD@468..470 "is"
                    WHITESPACE@470..471 " "
                    WORD@471..476 "given"
                    WHITESPACE@476..477 " "
                    WORD@477..479 "in"
                    WHITESPACE@479..480 " "
                    WORD@480..483 "the"
                    WHITESPACE@483..484 " "
                    COMMAND@484..491
                      COMMAND_NAME@484..491 "\\texttt"
                    CURLY_GROUP@491..499
                      L_CURLY@491..492 "{"
                      WORD@492..498 "number"
                      R_CURLY@498..499 "}"
                    WHITESPACE@499..500 " "
                    WORD@500..506 "field."
                    WHITESPACE@506..507 " "
                    WORD@507..511 "Note"
                    WHITESPACE@511..532 "\n                    "
                    WORD@532..535 "the"
                    WHITESPACE@535..536 " "
                    WORD@536..542 "format"
                    WHITESPACE@542..543 " "
                    WORD@543..545 "of"
                    WHITESPACE@545..546 " "
                    WORD@546..549 "the"
                    WHITESPACE@549..550 " "
                    COMMAND@550..557
                      COMMAND_NAME@550..557 "\\texttt"
                    CURLY_GROUP@557..567
                      L_CURLY@557..558 "{"
                      WORD@558..566 "location"
                      R_CURLY@566..567 "}"
                    WHITESPACE@567..568 " "
                    WORD@568..573 "field"
                    WHITESPACE@573..574 " "
                    WORD@574..576 "in"
                    WHITESPACE@576..577 " "
                    WORD@577..580 "the"
                    WHITESPACE@580..581 " "
                    WORD@581..589 "database"
                    WHITESPACE@589..610 "\n                    "
                    WORD@610..615 "file."
                    WHITESPACE@615..616 " "
                    WORD@616..623 "Compare"
                    WHITESPACE@623..624 " "
                    COMMAND@624..631
                      COMMAND_NAME@624..631 "\\texttt"
                    CURLY_GROUP@631..643
                      L_CURLY@631..632 "{"
                      WORD@632..642 "laufenberg"
                      R_CURLY@642..643 "}"
                    COMMA@643..644 ","
                    WHITESPACE@644..645 " "
                    COMMAND@645..652
                      COMMAND_NAME@645..652 "\\texttt"
                    CURLY_GROUP@652..660
                      L_CURLY@652..653 "{"
                      WORD@653..659 "sorace"
                      R_CURLY@659..660 "}"
                    COMMA@660..661 ","
                    WHITESPACE@661..662 " "
                    WORD@662..665 "and"
                    WHITESPACE@665..686 "\n                    "
                    COMMAND@686..693
                      COMMAND_NAME@686..693 "\\texttt"
                    CURLY_GROUP@693..702
                      L_CURLY@693..694 "{"
                      WORD@694..701 "kowalik"
                      R_CURLY@701..702 "}"
                    R_CURLY@702..703 "}"
                  COMMA@703..704 ","
                WHITESPACE@704..705 "\n"
                R_DELIM@705..706 "}"

        "#]],
    );
}

#[test]
fn test_averroes_1998() {
    check(
        r#"@book{averroes/bland,
    author       = {Averroes},
    title        = {The Epistle on the Possibility of Conjunction with the Active
                    Intellect by {Ibn Rushd} with the Commentary of {Moses Narboni}},
    date         = 1982,
    editor       = {Bland, Kalman P.},
    translator   = {Bland, Kalman P.},
    series       = {Moreshet: Studies in {Jewish} History, Literature and Thought},
    number       = 7,
    publisher    = {Jewish Theological Seminary of America},
    location     = {New York},
    keywords     = {primary},
    langid       = {english},
    langidopts   = {variant=american},
    indextitle   = {Epistle on the Possibility of Conjunction, The},
    shorttitle   = {Possibility of Conjunction},
    annotation   = {A \texttt{book} entry with a \texttt{series} and a
                    \texttt{number}. Note the concatenation of the \texttt{editor}
                    and \texttt{translator} fields as well as the
                    \texttt{indextitle} field},
}"#,
        expect![[r#"
            ROOT@0..1008
              ENTRY@0..1008
                TYPE@0..5 "@book"
                L_DELIM@5..6 "{"
                NAME@6..20 "averroes/bland"
                COMMA@20..21 ","
                WHITESPACE@21..26 "\n    "
                FIELD@26..52
                  NAME@26..32 "author"
                  WHITESPACE@32..39 "       "
                  EQ@39..40 "="
                  WHITESPACE@40..41 " "
                  CURLY_GROUP@41..51
                    L_CURLY@41..42 "{"
                    WORD@42..50 "Averroes"
                    R_CURLY@50..51 "}"
                  COMMA@51..52 ","
                WHITESPACE@52..57 "\n    "
                FIELD@57..220
                  NAME@57..62 "title"
                  WHITESPACE@62..70 "        "
                  EQ@70..71 "="
                  WHITESPACE@71..72 " "
                  CURLY_GROUP@72..219
                    L_CURLY@72..73 "{"
                    WORD@73..76 "The"
                    WHITESPACE@76..77 " "
                    WORD@77..84 "Epistle"
                    WHITESPACE@84..85 " "
                    WORD@85..87 "on"
                    WHITESPACE@87..88 " "
                    WORD@88..91 "the"
                    WHITESPACE@91..92 " "
                    WORD@92..103 "Possibility"
                    WHITESPACE@103..104 " "
                    WORD@104..106 "of"
                    WHITESPACE@106..107 " "
                    WORD@107..118 "Conjunction"
                    WHITESPACE@118..119 " "
                    WORD@119..123 "with"
                    WHITESPACE@123..124 " "
                    WORD@124..127 "the"
                    WHITESPACE@127..128 " "
                    WORD@128..134 "Active"
                    WHITESPACE@134..155 "\n                    "
                    WORD@155..164 "Intellect"
                    WHITESPACE@164..165 " "
                    WORD@165..167 "by"
                    WHITESPACE@167..168 " "
                    CURLY_GROUP@168..179
                      L_CURLY@168..169 "{"
                      WORD@169..172 "Ibn"
                      WHITESPACE@172..173 " "
                      WORD@173..178 "Rushd"
                      R_CURLY@178..179 "}"
                    WHITESPACE@179..180 " "
                    WORD@180..184 "with"
                    WHITESPACE@184..185 " "
                    WORD@185..188 "the"
                    WHITESPACE@188..189 " "
                    WORD@189..199 "Commentary"
                    WHITESPACE@199..200 " "
                    WORD@200..202 "of"
                    WHITESPACE@202..203 " "
                    CURLY_GROUP@203..218
                      L_CURLY@203..204 "{"
                      WORD@204..209 "Moses"
                      WHITESPACE@209..210 " "
                      WORD@210..217 "Narboni"
                      R_CURLY@217..218 "}"
                    R_CURLY@218..219 "}"
                  COMMA@219..220 ","
                WHITESPACE@220..225 "\n    "
                FIELD@225..245
                  NAME@225..229 "date"
                  WHITESPACE@229..238 "         "
                  EQ@238..239 "="
                  WHITESPACE@239..240 " "
                  LITERAL@240..244
                    INTEGER@240..244 "1982"
                  COMMA@244..245 ","
                WHITESPACE@245..250 "\n    "
                FIELD@250..284
                  NAME@250..256 "editor"
                  WHITESPACE@256..263 "       "
                  EQ@263..264 "="
                  WHITESPACE@264..265 " "
                  CURLY_GROUP@265..283
                    L_CURLY@265..266 "{"
                    WORD@266..271 "Bland"
                    COMMA@271..272 ","
                    WHITESPACE@272..273 " "
                    WORD@273..279 "Kalman"
                    WHITESPACE@279..280 " "
                    WORD@280..282 "P."
                    R_CURLY@282..283 "}"
                  COMMA@283..284 ","
                WHITESPACE@284..289 "\n    "
                FIELD@289..323
                  NAME@289..299 "translator"
                  WHITESPACE@299..302 "   "
                  EQ@302..303 "="
                  WHITESPACE@303..304 " "
                  CURLY_GROUP@304..322
                    L_CURLY@304..305 "{"
                    WORD@305..310 "Bland"
                    COMMA@310..311 ","
                    WHITESPACE@311..312 " "
                    WORD@312..318 "Kalman"
                    WHITESPACE@318..319 " "
                    WORD@319..321 "P."
                    R_CURLY@321..322 "}"
                  COMMA@322..323 ","
                WHITESPACE@323..328 "\n    "
                FIELD@328..407
                  NAME@328..334 "series"
                  WHITESPACE@334..341 "       "
                  EQ@341..342 "="
                  WHITESPACE@342..343 " "
                  CURLY_GROUP@343..406
                    L_CURLY@343..344 "{"
                    WORD@344..353 "Moreshet:"
                    WHITESPACE@353..354 " "
                    WORD@354..361 "Studies"
                    WHITESPACE@361..362 " "
                    WORD@362..364 "in"
                    WHITESPACE@364..365 " "
                    CURLY_GROUP@365..373
                      L_CURLY@365..366 "{"
                      WORD@366..372 "Jewish"
                      R_CURLY@372..373 "}"
                    WHITESPACE@373..374 " "
                    WORD@374..381 "History"
                    COMMA@381..382 ","
                    WHITESPACE@382..383 " "
                    WORD@383..393 "Literature"
                    WHITESPACE@393..394 " "
                    WORD@394..397 "and"
                    WHITESPACE@397..398 " "
                    WORD@398..405 "Thought"
                    R_CURLY@405..406 "}"
                  COMMA@406..407 ","
                WHITESPACE@407..412 "\n    "
                FIELD@412..429
                  NAME@412..418 "number"
                  WHITESPACE@418..425 "       "
                  EQ@425..426 "="
                  WHITESPACE@426..427 " "
                  LITERAL@427..428
                    INTEGER@427..428 "7"
                  COMMA@428..429 ","
                WHITESPACE@429..434 "\n    "
                FIELD@434..490
                  NAME@434..443 "publisher"
                  WHITESPACE@443..447 "    "
                  EQ@447..448 "="
                  WHITESPACE@448..449 " "
                  CURLY_GROUP@449..489
                    L_CURLY@449..450 "{"
                    WORD@450..456 "Jewish"
                    WHITESPACE@456..457 " "
                    WORD@457..468 "Theological"
                    WHITESPACE@468..469 " "
                    WORD@469..477 "Seminary"
                    WHITESPACE@477..478 " "
                    WORD@478..480 "of"
                    WHITESPACE@480..481 " "
                    WORD@481..488 "America"
                    R_CURLY@488..489 "}"
                  COMMA@489..490 ","
                WHITESPACE@490..495 "\n    "
                FIELD@495..521
                  NAME@495..503 "location"
                  WHITESPACE@503..508 "     "
                  EQ@508..509 "="
                  WHITESPACE@509..510 " "
                  CURLY_GROUP@510..520
                    L_CURLY@510..511 "{"
                    WORD@511..514 "New"
                    WHITESPACE@514..515 " "
                    WORD@515..519 "York"
                    R_CURLY@519..520 "}"
                  COMMA@520..521 ","
                WHITESPACE@521..526 "\n    "
                FIELD@526..551
                  NAME@526..534 "keywords"
                  WHITESPACE@534..539 "     "
                  EQ@539..540 "="
                  WHITESPACE@540..541 " "
                  CURLY_GROUP@541..550
                    L_CURLY@541..542 "{"
                    WORD@542..549 "primary"
                    R_CURLY@549..550 "}"
                  COMMA@550..551 ","
                WHITESPACE@551..556 "\n    "
                FIELD@556..581
                  NAME@556..562 "langid"
                  WHITESPACE@562..569 "       "
                  EQ@569..570 "="
                  WHITESPACE@570..571 " "
                  CURLY_GROUP@571..580
                    L_CURLY@571..572 "{"
                    WORD@572..579 "english"
                    R_CURLY@579..580 "}"
                  COMMA@580..581 ","
                WHITESPACE@581..586 "\n    "
                FIELD@586..620
                  NAME@586..596 "langidopts"
                  WHITESPACE@596..599 "   "
                  EQ@599..600 "="
                  WHITESPACE@600..601 " "
                  CURLY_GROUP@601..619
                    L_CURLY@601..602 "{"
                    WORD@602..618 "variant=american"
                    R_CURLY@618..619 "}"
                  COMMA@619..620 ","
                WHITESPACE@620..625 "\n    "
                FIELD@625..689
                  NAME@625..635 "indextitle"
                  WHITESPACE@635..638 "   "
                  EQ@638..639 "="
                  WHITESPACE@639..640 " "
                  CURLY_GROUP@640..688
                    L_CURLY@640..641 "{"
                    WORD@641..648 "Epistle"
                    WHITESPACE@648..649 " "
                    WORD@649..651 "on"
                    WHITESPACE@651..652 " "
                    WORD@652..655 "the"
                    WHITESPACE@655..656 " "
                    WORD@656..667 "Possibility"
                    WHITESPACE@667..668 " "
                    WORD@668..670 "of"
                    WHITESPACE@670..671 " "
                    WORD@671..682 "Conjunction"
                    COMMA@682..683 ","
                    WHITESPACE@683..684 " "
                    WORD@684..687 "The"
                    R_CURLY@687..688 "}"
                  COMMA@688..689 ","
                WHITESPACE@689..694 "\n    "
                FIELD@694..738
                  NAME@694..704 "shorttitle"
                  WHITESPACE@704..707 "   "
                  EQ@707..708 "="
                  WHITESPACE@708..709 " "
                  CURLY_GROUP@709..737
                    L_CURLY@709..710 "{"
                    WORD@710..721 "Possibility"
                    WHITESPACE@721..722 " "
                    WORD@722..724 "of"
                    WHITESPACE@724..725 " "
                    WORD@725..736 "Conjunction"
                    R_CURLY@736..737 "}"
                  COMMA@737..738 ","
                WHITESPACE@738..743 "\n    "
                FIELD@743..1006
                  NAME@743..753 "annotation"
                  WHITESPACE@753..756 "   "
                  EQ@756..757 "="
                  WHITESPACE@757..758 " "
                  CURLY_GROUP@758..1005
                    L_CURLY@758..759 "{"
                    WORD@759..760 "A"
                    WHITESPACE@760..761 " "
                    COMMAND@761..768
                      COMMAND_NAME@761..768 "\\texttt"
                    CURLY_GROUP@768..774
                      L_CURLY@768..769 "{"
                      WORD@769..773 "book"
                      R_CURLY@773..774 "}"
                    WHITESPACE@774..775 " "
                    WORD@775..780 "entry"
                    WHITESPACE@780..781 " "
                    WORD@781..785 "with"
                    WHITESPACE@785..786 " "
                    WORD@786..787 "a"
                    WHITESPACE@787..788 " "
                    COMMAND@788..795
                      COMMAND_NAME@788..795 "\\texttt"
                    CURLY_GROUP@795..803
                      L_CURLY@795..796 "{"
                      WORD@796..802 "series"
                      R_CURLY@802..803 "}"
                    WHITESPACE@803..804 " "
                    WORD@804..807 "and"
                    WHITESPACE@807..808 " "
                    WORD@808..809 "a"
                    WHITESPACE@809..830 "\n                    "
                    COMMAND@830..837
                      COMMAND_NAME@830..837 "\\texttt"
                    CURLY_GROUP@837..845
                      L_CURLY@837..838 "{"
                      WORD@838..844 "number"
                      R_CURLY@844..845 "}"
                    WORD@845..846 "."
                    WHITESPACE@846..847 " "
                    WORD@847..851 "Note"
                    WHITESPACE@851..852 " "
                    WORD@852..855 "the"
                    WHITESPACE@855..856 " "
                    WORD@856..869 "concatenation"
                    WHITESPACE@869..870 " "
                    WORD@870..872 "of"
                    WHITESPACE@872..873 " "
                    WORD@873..876 "the"
                    WHITESPACE@876..877 " "
                    COMMAND@877..884
                      COMMAND_NAME@877..884 "\\texttt"
                    CURLY_GROUP@884..892
                      L_CURLY@884..885 "{"
                      WORD@885..891 "editor"
                      R_CURLY@891..892 "}"
                    WHITESPACE@892..913 "\n                    "
                    WORD@913..916 "and"
                    WHITESPACE@916..917 " "
                    COMMAND@917..924
                      COMMAND_NAME@917..924 "\\texttt"
                    CURLY_GROUP@924..936
                      L_CURLY@924..925 "{"
                      WORD@925..935 "translator"
                      R_CURLY@935..936 "}"
                    WHITESPACE@936..937 " "
                    WORD@937..943 "fields"
                    WHITESPACE@943..944 " "
                    WORD@944..946 "as"
                    WHITESPACE@946..947 " "
                    WORD@947..951 "well"
                    WHITESPACE@951..952 " "
                    WORD@952..954 "as"
                    WHITESPACE@954..955 " "
                    WORD@955..958 "the"
                    WHITESPACE@958..979 "\n                    "
                    COMMAND@979..986
                      COMMAND_NAME@979..986 "\\texttt"
                    CURLY_GROUP@986..998
                      L_CURLY@986..987 "{"
                      WORD@987..997 "indextitle"
                      R_CURLY@997..998 "}"
                    WHITESPACE@998..999 " "
                    WORD@999..1004 "field"
                    R_CURLY@1004..1005 "}"
                  COMMA@1005..1006 ","
                WHITESPACE@1006..1007 "\n"
                R_DELIM@1007..1008 "}"

        "#]],
    );
}

#[test]
fn test_betram_1996() {
    check(
        r#"@string{jams    = {J.~Amer. Math. Soc.}}

@article{bertram,
    author       = {Bertram, Aaron and Wentworth, Richard},
    title        = {Gromov invariants for holomorphic maps on {Riemann} surfaces},
    journaltitle = jams,
    date         = 1996,
    volume       = 9,
    number       = 2,
    pages        = {529-571},
    langid       = {english},
    langidopts   = {variant=american},
    shorttitle   = {Gromov invariants},
    annotation   = {An \texttt{article} entry with a \texttt{volume} and a
                    \texttt{number} field},
}"#,
        expect![[r#"
            ROOT@0..556
              STRING@0..40
                TYPE@0..7 "@string"
                L_DELIM@7..8 "{"
                NAME@8..12 "jams"
                WHITESPACE@12..16 "    "
                EQ@16..17 "="
                WHITESPACE@17..18 " "
                CURLY_GROUP@18..39
                  L_CURLY@18..19 "{"
                  WORD@19..21 "J."
                  NBSP@21..22 "~"
                  WORD@22..27 "Amer."
                  WHITESPACE@27..28 " "
                  WORD@28..33 "Math."
                  WHITESPACE@33..34 " "
                  WORD@34..38 "Soc."
                  R_CURLY@38..39 "}"
                R_DELIM@39..40 "}"
              JUNK@40..42 "\n\n"
              ENTRY@42..556
                TYPE@42..50 "@article"
                L_DELIM@50..51 "{"
                NAME@51..58 "bertram"
                COMMA@58..59 ","
                WHITESPACE@59..64 "\n    "
                FIELD@64..119
                  NAME@64..70 "author"
                  WHITESPACE@70..77 "       "
                  EQ@77..78 "="
                  WHITESPACE@78..79 " "
                  CURLY_GROUP@79..118
                    L_CURLY@79..80 "{"
                    WORD@80..87 "Bertram"
                    COMMA@87..88 ","
                    WHITESPACE@88..89 " "
                    WORD@89..94 "Aaron"
                    WHITESPACE@94..95 " "
                    WORD@95..98 "and"
                    WHITESPACE@98..99 " "
                    WORD@99..108 "Wentworth"
                    COMMA@108..109 ","
                    WHITESPACE@109..110 " "
                    WORD@110..117 "Richard"
                    R_CURLY@117..118 "}"
                  COMMA@118..119 ","
                WHITESPACE@119..124 "\n    "
                FIELD@124..202
                  NAME@124..129 "title"
                  WHITESPACE@129..137 "        "
                  EQ@137..138 "="
                  WHITESPACE@138..139 " "
                  CURLY_GROUP@139..201
                    L_CURLY@139..140 "{"
                    WORD@140..146 "Gromov"
                    WHITESPACE@146..147 " "
                    WORD@147..157 "invariants"
                    WHITESPACE@157..158 " "
                    WORD@158..161 "for"
                    WHITESPACE@161..162 " "
                    WORD@162..173 "holomorphic"
                    WHITESPACE@173..174 " "
                    WORD@174..178 "maps"
                    WHITESPACE@178..179 " "
                    WORD@179..181 "on"
                    WHITESPACE@181..182 " "
                    CURLY_GROUP@182..191
                      L_CURLY@182..183 "{"
                      WORD@183..190 "Riemann"
                      R_CURLY@190..191 "}"
                    WHITESPACE@191..192 " "
                    WORD@192..200 "surfaces"
                    R_CURLY@200..201 "}"
                  COMMA@201..202 ","
                WHITESPACE@202..207 "\n    "
                FIELD@207..227
                  NAME@207..219 "journaltitle"
                  WHITESPACE@219..220 " "
                  EQ@220..221 "="
                  WHITESPACE@221..222 " "
                  LITERAL@222..226
                    NAME@222..226 "jams"
                  COMMA@226..227 ","
                WHITESPACE@227..232 "\n    "
                FIELD@232..252
                  NAME@232..236 "date"
                  WHITESPACE@236..245 "         "
                  EQ@245..246 "="
                  WHITESPACE@246..247 " "
                  LITERAL@247..251
                    INTEGER@247..251 "1996"
                  COMMA@251..252 ","
                WHITESPACE@252..257 "\n    "
                FIELD@257..274
                  NAME@257..263 "volume"
                  WHITESPACE@263..270 "       "
                  EQ@270..271 "="
                  WHITESPACE@271..272 " "
                  LITERAL@272..273
                    INTEGER@272..273 "9"
                  COMMA@273..274 ","
                WHITESPACE@274..279 "\n    "
                FIELD@279..296
                  NAME@279..285 "number"
                  WHITESPACE@285..292 "       "
                  EQ@292..293 "="
                  WHITESPACE@293..294 " "
                  LITERAL@294..295
                    INTEGER@294..295 "2"
                  COMMA@295..296 ","
                WHITESPACE@296..301 "\n    "
                FIELD@301..326
                  NAME@301..306 "pages"
                  WHITESPACE@306..314 "        "
                  EQ@314..315 "="
                  WHITESPACE@315..316 " "
                  CURLY_GROUP@316..325
                    L_CURLY@316..317 "{"
                    WORD@317..324 "529-571"
                    R_CURLY@324..325 "}"
                  COMMA@325..326 ","
                WHITESPACE@326..331 "\n    "
                FIELD@331..356
                  NAME@331..337 "langid"
                  WHITESPACE@337..344 "       "
                  EQ@344..345 "="
                  WHITESPACE@345..346 " "
                  CURLY_GROUP@346..355
                    L_CURLY@346..347 "{"
                    WORD@347..354 "english"
                    R_CURLY@354..355 "}"
                  COMMA@355..356 ","
                WHITESPACE@356..361 "\n    "
                FIELD@361..395
                  NAME@361..371 "langidopts"
                  WHITESPACE@371..374 "   "
                  EQ@374..375 "="
                  WHITESPACE@375..376 " "
                  CURLY_GROUP@376..394
                    L_CURLY@376..377 "{"
                    WORD@377..393 "variant=american"
                    R_CURLY@393..394 "}"
                  COMMA@394..395 ","
                WHITESPACE@395..400 "\n    "
                FIELD@400..435
                  NAME@400..410 "shorttitle"
                  WHITESPACE@410..413 "   "
                  EQ@413..414 "="
                  WHITESPACE@414..415 " "
                  CURLY_GROUP@415..434
                    L_CURLY@415..416 "{"
                    WORD@416..422 "Gromov"
                    WHITESPACE@422..423 " "
                    WORD@423..433 "invariants"
                    R_CURLY@433..434 "}"
                  COMMA@434..435 ","
                WHITESPACE@435..440 "\n    "
                FIELD@440..554
                  NAME@440..450 "annotation"
                  WHITESPACE@450..453 "   "
                  EQ@453..454 "="
                  WHITESPACE@454..455 " "
                  CURLY_GROUP@455..553
                    L_CURLY@455..456 "{"
                    WORD@456..458 "An"
                    WHITESPACE@458..459 " "
                    COMMAND@459..466
                      COMMAND_NAME@459..466 "\\texttt"
                    CURLY_GROUP@466..475
                      L_CURLY@466..467 "{"
                      WORD@467..474 "article"
                      R_CURLY@474..475 "}"
                    WHITESPACE@475..476 " "
                    WORD@476..481 "entry"
                    WHITESPACE@481..482 " "
                    WORD@482..486 "with"
                    WHITESPACE@486..487 " "
                    WORD@487..488 "a"
                    WHITESPACE@488..489 " "
                    COMMAND@489..496
                      COMMAND_NAME@489..496 "\\texttt"
                    CURLY_GROUP@496..504
                      L_CURLY@496..497 "{"
                      WORD@497..503 "volume"
                      R_CURLY@503..504 "}"
                    WHITESPACE@504..505 " "
                    WORD@505..508 "and"
                    WHITESPACE@508..509 " "
                    WORD@509..510 "a"
                    WHITESPACE@510..531 "\n                    "
                    COMMAND@531..538
                      COMMAND_NAME@531..538 "\\texttt"
                    CURLY_GROUP@538..546
                      L_CURLY@538..539 "{"
                      WORD@539..545 "number"
                      R_CURLY@545..546 "}"
                    WHITESPACE@546..547 " "
                    WORD@547..552 "field"
                    R_CURLY@552..553 "}"
                  COMMA@553..554 ","
                WHITESPACE@554..555 "\n"
                R_DELIM@555..556 "}"

        "#]],
    );
}

#[test]
fn test_blom_2021() {
    check(
        r#"@article{DBLP:journals/corr/abs-2107-11903,
    author    = {Michelle L. Blom and
                    Jurlind Budurushi and
                    Ronald L. Rivest and
                    Philip B. Stark and
                    Peter J. Stuckey and
                    Vanessa Teague and
                    Damjan Vukcevic},
    title     = {Assertion-based Approaches to Auditing Complex Elections, with application
                    to party-list proportional elections},
    journal   = {CoRR},
    volume    = {abs/2107.11903},
    year      = {2021},
    url       = {https://arxiv.org/abs/2107.11903},
    eprinttype = {arXiv},
    eprint    = {2107.11903},
    timestamp = {Thu, 29 Jul 2021 16:14:15 +0200},
    biburl    = {https://dblp.org/rec/journals/corr/abs-2107-11903.bib},
    bibsource = {dblp computer science bibliography, https://dblp.org}
}"#,
        expect![[r#"
            ROOT@0..860
              ENTRY@0..860
                TYPE@0..8 "@article"
                L_DELIM@8..9 "{"
                NAME@9..42 "DBLP:journals/corr/ab ..."
                COMMA@42..43 ","
                WHITESPACE@43..48 "\n    "
                FIELD@48..322
                  NAME@48..54 "author"
                  WHITESPACE@54..58 "    "
                  EQ@58..59 "="
                  WHITESPACE@59..60 " "
                  CURLY_GROUP@60..321
                    L_CURLY@60..61 "{"
                    WORD@61..69 "Michelle"
                    WHITESPACE@69..70 " "
                    WORD@70..72 "L."
                    WHITESPACE@72..73 " "
                    WORD@73..77 "Blom"
                    WHITESPACE@77..78 " "
                    WORD@78..81 "and"
                    WHITESPACE@81..102 "\n                    "
                    WORD@102..109 "Jurlind"
                    WHITESPACE@109..110 " "
                    WORD@110..119 "Budurushi"
                    WHITESPACE@119..120 " "
                    WORD@120..123 "and"
                    WHITESPACE@123..144 "\n                    "
                    WORD@144..150 "Ronald"
                    WHITESPACE@150..151 " "
                    WORD@151..153 "L."
                    WHITESPACE@153..154 " "
                    WORD@154..160 "Rivest"
                    WHITESPACE@160..161 " "
                    WORD@161..164 "and"
                    WHITESPACE@164..185 "\n                    "
                    WORD@185..191 "Philip"
                    WHITESPACE@191..192 " "
                    WORD@192..194 "B."
                    WHITESPACE@194..195 " "
                    WORD@195..200 "Stark"
                    WHITESPACE@200..201 " "
                    WORD@201..204 "and"
                    WHITESPACE@204..225 "\n                    "
                    WORD@225..230 "Peter"
                    WHITESPACE@230..231 " "
                    WORD@231..233 "J."
                    WHITESPACE@233..234 " "
                    WORD@234..241 "Stuckey"
                    WHITESPACE@241..242 " "
                    WORD@242..245 "and"
                    WHITESPACE@245..266 "\n                    "
                    WORD@266..273 "Vanessa"
                    WHITESPACE@273..274 " "
                    WORD@274..280 "Teague"
                    WHITESPACE@280..281 " "
                    WORD@281..284 "and"
                    WHITESPACE@284..305 "\n                    "
                    WORD@305..311 "Damjan"
                    WHITESPACE@311..312 " "
                    WORD@312..320 "Vukcevic"
                    R_CURLY@320..321 "}"
                  COMMA@321..322 ","
                WHITESPACE@322..327 "\n    "
                FIELD@327..473
                  NAME@327..332 "title"
                  WHITESPACE@332..337 "     "
                  EQ@337..338 "="
                  WHITESPACE@338..339 " "
                  CURLY_GROUP@339..472
                    L_CURLY@339..340 "{"
                    WORD@340..355 "Assertion-based"
                    WHITESPACE@355..356 " "
                    WORD@356..366 "Approaches"
                    WHITESPACE@366..367 " "
                    WORD@367..369 "to"
                    WHITESPACE@369..370 " "
                    WORD@370..378 "Auditing"
                    WHITESPACE@378..379 " "
                    WORD@379..386 "Complex"
                    WHITESPACE@386..387 " "
                    WORD@387..396 "Elections"
                    COMMA@396..397 ","
                    WHITESPACE@397..398 " "
                    WORD@398..402 "with"
                    WHITESPACE@402..403 " "
                    WORD@403..414 "application"
                    WHITESPACE@414..435 "\n                    "
                    WORD@435..437 "to"
                    WHITESPACE@437..438 " "
                    WORD@438..448 "party-list"
                    WHITESPACE@448..449 " "
                    WORD@449..461 "proportional"
                    WHITESPACE@461..462 " "
                    WORD@462..471 "elections"
                    R_CURLY@471..472 "}"
                  COMMA@472..473 ","
                WHITESPACE@473..478 "\n    "
                FIELD@478..497
                  NAME@478..485 "journal"
                  WHITESPACE@485..488 "   "
                  EQ@488..489 "="
                  WHITESPACE@489..490 " "
                  CURLY_GROUP@490..496
                    L_CURLY@490..491 "{"
                    WORD@491..495 "CoRR"
                    R_CURLY@495..496 "}"
                  COMMA@496..497 ","
                WHITESPACE@497..502 "\n    "
                FIELD@502..531
                  NAME@502..508 "volume"
                  WHITESPACE@508..512 "    "
                  EQ@512..513 "="
                  WHITESPACE@513..514 " "
                  CURLY_GROUP@514..530
                    L_CURLY@514..515 "{"
                    WORD@515..529 "abs/2107.11903"
                    R_CURLY@529..530 "}"
                  COMMA@530..531 ","
                WHITESPACE@531..536 "\n    "
                FIELD@536..555
                  NAME@536..540 "year"
                  WHITESPACE@540..546 "      "
                  EQ@546..547 "="
                  WHITESPACE@547..548 " "
                  CURLY_GROUP@548..554
                    L_CURLY@548..549 "{"
                    INTEGER@549..553 "2021"
                    R_CURLY@553..554 "}"
                  COMMA@554..555 ","
                WHITESPACE@555..560 "\n    "
                FIELD@560..607
                  NAME@560..563 "url"
                  WHITESPACE@563..570 "       "
                  EQ@570..571 "="
                  WHITESPACE@571..572 " "
                  CURLY_GROUP@572..606
                    L_CURLY@572..573 "{"
                    WORD@573..605 "https://arxiv.org/abs ..."
                    R_CURLY@605..606 "}"
                  COMMA@606..607 ","
                WHITESPACE@607..612 "\n    "
                FIELD@612..633
                  NAME@612..622 "eprinttype"
                  WHITESPACE@622..623 " "
                  EQ@623..624 "="
                  WHITESPACE@624..625 " "
                  CURLY_GROUP@625..632
                    L_CURLY@625..626 "{"
                    WORD@626..631 "arXiv"
                    R_CURLY@631..632 "}"
                  COMMA@632..633 ","
                WHITESPACE@633..638 "\n    "
                FIELD@638..663
                  NAME@638..644 "eprint"
                  WHITESPACE@644..648 "    "
                  EQ@648..649 "="
                  WHITESPACE@649..650 " "
                  CURLY_GROUP@650..662
                    L_CURLY@650..651 "{"
                    WORD@651..661 "2107.11903"
                    R_CURLY@661..662 "}"
                  COMMA@662..663 ","
                WHITESPACE@663..668 "\n    "
                FIELD@668..714
                  NAME@668..677 "timestamp"
                  WHITESPACE@677..678 " "
                  EQ@678..679 "="
                  WHITESPACE@679..680 " "
                  CURLY_GROUP@680..713
                    L_CURLY@680..681 "{"
                    WORD@681..684 "Thu"
                    COMMA@684..685 ","
                    WHITESPACE@685..686 " "
                    INTEGER@686..688 "29"
                    WHITESPACE@688..689 " "
                    WORD@689..692 "Jul"
                    WHITESPACE@692..693 " "
                    INTEGER@693..697 "2021"
                    WHITESPACE@697..698 " "
                    WORD@698..706 "16:14:15"
                    WHITESPACE@706..707 " "
                    WORD@707..712 "+0200"
                    R_CURLY@712..713 "}"
                  COMMA@713..714 ","
                WHITESPACE@714..719 "\n    "
                FIELD@719..787
                  NAME@719..725 "biburl"
                  WHITESPACE@725..729 "    "
                  EQ@729..730 "="
                  WHITESPACE@730..731 " "
                  CURLY_GROUP@731..786
                    L_CURLY@731..732 "{"
                    WORD@732..785 "https://dblp.org/rec/ ..."
                    R_CURLY@785..786 "}"
                  COMMA@786..787 ","
                WHITESPACE@787..792 "\n    "
                FIELD@792..859
                  NAME@792..801 "bibsource"
                  WHITESPACE@801..802 " "
                  EQ@802..803 "="
                  WHITESPACE@803..804 " "
                  CURLY_GROUP@804..858
                    L_CURLY@804..805 "{"
                    WORD@805..809 "dblp"
                    WHITESPACE@809..810 " "
                    WORD@810..818 "computer"
                    WHITESPACE@818..819 " "
                    WORD@819..826 "science"
                    WHITESPACE@826..827 " "
                    WORD@827..839 "bibliography"
                    COMMA@839..840 ","
                    WHITESPACE@840..841 " "
                    WORD@841..857 "https://dblp.org"
                    R_CURLY@857..858 "}"
                  WHITESPACE@858..859 "\n"
                R_DELIM@859..860 "}"

        "#]],
    );
}

#[test]
fn test_combi_2004() {
    check(
        r#"@inproceedings{10.1145/967900.968040,
    author = {Combi, Carlo and Pozzi, Giuseppe},
    title = {Architectures for a Temporal Workflow Management System},
    year = {2004},
    isbn = {1581138121},
    publisher = {Association for Computing Machinery},
    address = {New York, NY, USA},
    url = {https://doi.org/10.1145/967900.968040},
    doi = {10.1145/967900.968040},
    booktitle = {Proceedings of the 2004 ACM Symposium on Applied Computing},
    pages = {659-666},
    numpages = {8},
    keywords = {active DBMS, temporal DBMS, workflow management system - WfMS, temporal workflow management system},
    location = {Nicosia, Cyprus},
    series = {SAC '04}
}"#,
        expect![[r#"
            ROOT@0..674
              ENTRY@0..674
                TYPE@0..14 "@inproceedings"
                L_DELIM@14..15 "{"
                NAME@15..36 "10.1145/967900.968040"
                COMMA@36..37 ","
                WHITESPACE@37..42 "\n    "
                FIELD@42..86
                  NAME@42..48 "author"
                  WHITESPACE@48..49 " "
                  EQ@49..50 "="
                  WHITESPACE@50..51 " "
                  CURLY_GROUP@51..85
                    L_CURLY@51..52 "{"
                    WORD@52..57 "Combi"
                    COMMA@57..58 ","
                    WHITESPACE@58..59 " "
                    WORD@59..64 "Carlo"
                    WHITESPACE@64..65 " "
                    WORD@65..68 "and"
                    WHITESPACE@68..69 " "
                    WORD@69..74 "Pozzi"
                    COMMA@74..75 ","
                    WHITESPACE@75..76 " "
                    WORD@76..84 "Giuseppe"
                    R_CURLY@84..85 "}"
                  COMMA@85..86 ","
                WHITESPACE@86..91 "\n    "
                FIELD@91..157
                  NAME@91..96 "title"
                  WHITESPACE@96..97 " "
                  EQ@97..98 "="
                  WHITESPACE@98..99 " "
                  CURLY_GROUP@99..156
                    L_CURLY@99..100 "{"
                    WORD@100..113 "Architectures"
                    WHITESPACE@113..114 " "
                    WORD@114..117 "for"
                    WHITESPACE@117..118 " "
                    WORD@118..119 "a"
                    WHITESPACE@119..120 " "
                    WORD@120..128 "Temporal"
                    WHITESPACE@128..129 " "
                    WORD@129..137 "Workflow"
                    WHITESPACE@137..138 " "
                    WORD@138..148 "Management"
                    WHITESPACE@148..149 " "
                    WORD@149..155 "System"
                    R_CURLY@155..156 "}"
                  COMMA@156..157 ","
                WHITESPACE@157..162 "\n    "
                FIELD@162..176
                  NAME@162..166 "year"
                  WHITESPACE@166..167 " "
                  EQ@167..168 "="
                  WHITESPACE@168..169 " "
                  CURLY_GROUP@169..175
                    L_CURLY@169..170 "{"
                    INTEGER@170..174 "2004"
                    R_CURLY@174..175 "}"
                  COMMA@175..176 ","
                WHITESPACE@176..181 "\n    "
                FIELD@181..201
                  NAME@181..185 "isbn"
                  WHITESPACE@185..186 " "
                  EQ@186..187 "="
                  WHITESPACE@187..188 " "
                  CURLY_GROUP@188..200
                    L_CURLY@188..189 "{"
                    INTEGER@189..199 "1581138121"
                    R_CURLY@199..200 "}"
                  COMMA@200..201 ","
                WHITESPACE@201..206 "\n    "
                FIELD@206..256
                  NAME@206..215 "publisher"
                  WHITESPACE@215..216 " "
                  EQ@216..217 "="
                  WHITESPACE@217..218 " "
                  CURLY_GROUP@218..255
                    L_CURLY@218..219 "{"
                    WORD@219..230 "Association"
                    WHITESPACE@230..231 " "
                    WORD@231..234 "for"
                    WHITESPACE@234..235 " "
                    WORD@235..244 "Computing"
                    WHITESPACE@244..245 " "
                    WORD@245..254 "Machinery"
                    R_CURLY@254..255 "}"
                  COMMA@255..256 ","
                WHITESPACE@256..261 "\n    "
                FIELD@261..291
                  NAME@261..268 "address"
                  WHITESPACE@268..269 " "
                  EQ@269..270 "="
                  WHITESPACE@270..271 " "
                  CURLY_GROUP@271..290
                    L_CURLY@271..272 "{"
                    WORD@272..275 "New"
                    WHITESPACE@275..276 " "
                    WORD@276..280 "York"
                    COMMA@280..281 ","
                    WHITESPACE@281..282 " "
                    WORD@282..284 "NY"
                    COMMA@284..285 ","
                    WHITESPACE@285..286 " "
                    WORD@286..289 "USA"
                    R_CURLY@289..290 "}"
                  COMMA@290..291 ","
                WHITESPACE@291..296 "\n    "
                FIELD@296..342
                  NAME@296..299 "url"
                  WHITESPACE@299..300 " "
                  EQ@300..301 "="
                  WHITESPACE@301..302 " "
                  CURLY_GROUP@302..341
                    L_CURLY@302..303 "{"
                    WORD@303..340 "https://doi.org/10.11 ..."
                    R_CURLY@340..341 "}"
                  COMMA@341..342 ","
                WHITESPACE@342..347 "\n    "
                FIELD@347..377
                  NAME@347..350 "doi"
                  WHITESPACE@350..351 " "
                  EQ@351..352 "="
                  WHITESPACE@352..353 " "
                  CURLY_GROUP@353..376
                    L_CURLY@353..354 "{"
                    WORD@354..375 "10.1145/967900.968040"
                    R_CURLY@375..376 "}"
                  COMMA@376..377 ","
                WHITESPACE@377..382 "\n    "
                FIELD@382..455
                  NAME@382..391 "booktitle"
                  WHITESPACE@391..392 " "
                  EQ@392..393 "="
                  WHITESPACE@393..394 " "
                  CURLY_GROUP@394..454
                    L_CURLY@394..395 "{"
                    WORD@395..406 "Proceedings"
                    WHITESPACE@406..407 " "
                    WORD@407..409 "of"
                    WHITESPACE@409..410 " "
                    WORD@410..413 "the"
                    WHITESPACE@413..414 " "
                    INTEGER@414..418 "2004"
                    WHITESPACE@418..419 " "
                    WORD@419..422 "ACM"
                    WHITESPACE@422..423 " "
                    WORD@423..432 "Symposium"
                    WHITESPACE@432..433 " "
                    WORD@433..435 "on"
                    WHITESPACE@435..436 " "
                    WORD@436..443 "Applied"
                    WHITESPACE@443..444 " "
                    WORD@444..453 "Computing"
                    R_CURLY@453..454 "}"
                  COMMA@454..455 ","
                WHITESPACE@455..460 "\n    "
                FIELD@460..478
                  NAME@460..465 "pages"
                  WHITESPACE@465..466 " "
                  EQ@466..467 "="
                  WHITESPACE@467..468 " "
                  CURLY_GROUP@468..477
                    L_CURLY@468..469 "{"
                    WORD@469..476 "659-666"
                    R_CURLY@476..477 "}"
                  COMMA@477..478 ","
                WHITESPACE@478..483 "\n    "
                FIELD@483..498
                  NAME@483..491 "numpages"
                  WHITESPACE@491..492 " "
                  EQ@492..493 "="
                  WHITESPACE@493..494 " "
                  CURLY_GROUP@494..497
                    L_CURLY@494..495 "{"
                    INTEGER@495..496 "8"
                    R_CURLY@496..497 "}"
                  COMMA@497..498 ","
                WHITESPACE@498..503 "\n    "
                FIELD@503..615
                  NAME@503..511 "keywords"
                  WHITESPACE@511..512 " "
                  EQ@512..513 "="
                  WHITESPACE@513..514 " "
                  CURLY_GROUP@514..614
                    L_CURLY@514..515 "{"
                    WORD@515..521 "active"
                    WHITESPACE@521..522 " "
                    WORD@522..526 "DBMS"
                    COMMA@526..527 ","
                    WHITESPACE@527..528 " "
                    WORD@528..536 "temporal"
                    WHITESPACE@536..537 " "
                    WORD@537..541 "DBMS"
                    COMMA@541..542 ","
                    WHITESPACE@542..543 " "
                    WORD@543..551 "workflow"
                    WHITESPACE@551..552 " "
                    WORD@552..562 "management"
                    WHITESPACE@562..563 " "
                    WORD@563..569 "system"
                    WHITESPACE@569..570 " "
                    WORD@570..571 "-"
                    WHITESPACE@571..572 " "
                    WORD@572..576 "WfMS"
                    COMMA@576..577 ","
                    WHITESPACE@577..578 " "
                    WORD@578..586 "temporal"
                    WHITESPACE@586..587 " "
                    WORD@587..595 "workflow"
                    WHITESPACE@595..596 " "
                    WORD@596..606 "management"
                    WHITESPACE@606..607 " "
                    WORD@607..613 "system"
                    R_CURLY@613..614 "}"
                  COMMA@614..615 ","
                WHITESPACE@615..620 "\n    "
                FIELD@620..649
                  NAME@620..628 "location"
                  WHITESPACE@628..629 " "
                  EQ@629..630 "="
                  WHITESPACE@630..631 " "
                  CURLY_GROUP@631..648
                    L_CURLY@631..632 "{"
                    WORD@632..639 "Nicosia"
                    COMMA@639..640 ","
                    WHITESPACE@640..641 " "
                    WORD@641..647 "Cyprus"
                    R_CURLY@647..648 "}"
                  COMMA@648..649 ","
                WHITESPACE@649..654 "\n    "
                FIELD@654..673
                  NAME@654..660 "series"
                  WHITESPACE@660..661 " "
                  EQ@661..662 "="
                  WHITESPACE@662..663 " "
                  CURLY_GROUP@663..672
                    L_CURLY@663..664 "{"
                    WORD@664..667 "SAC"
                    WHITESPACE@667..668 " "
                    WORD@668..671 "'04"
                    R_CURLY@671..672 "}"
                  WHITESPACE@672..673 "\n"
                R_DELIM@673..674 "}"

        "#]],
    );
}

#[test]
fn test_erwin_2007() {
    check(
        r#"@inproceedings{10.5555/1386993.1386994,
    author = {Erwin, Alva and Gopalan, Raj P. and Achuthan, N. R.},
    title = {A Bottom-up Projection Based Algorithm for Mining High Utility Itemsets},
    year = {2007},
    isbn = {9781920682651},
    publisher = {Australian Computer Society, Inc.},
    address = {AUS},
    booktitle = {Proceedings of the 2nd International Workshop on Integrating Artificial Intelligence and Data Mining - Volume 84},
    pages = {3-11},
    numpages = {9},
    keywords = {pattern growth, high utility itemset mining},
    location = {Gold Coast, Australia},
    series = {AIDM '07}
}"#,
        expect![[r#"
            ROOT@0..615
              ENTRY@0..615
                TYPE@0..14 "@inproceedings"
                L_DELIM@14..15 "{"
                NAME@15..38 "10.5555/1386993.1386994"
                COMMA@38..39 ","
                WHITESPACE@39..44 "\n    "
                FIELD@44..107
                  NAME@44..50 "author"
                  WHITESPACE@50..51 " "
                  EQ@51..52 "="
                  WHITESPACE@52..53 " "
                  CURLY_GROUP@53..106
                    L_CURLY@53..54 "{"
                    WORD@54..59 "Erwin"
                    COMMA@59..60 ","
                    WHITESPACE@60..61 " "
                    WORD@61..65 "Alva"
                    WHITESPACE@65..66 " "
                    WORD@66..69 "and"
                    WHITESPACE@69..70 " "
                    WORD@70..77 "Gopalan"
                    COMMA@77..78 ","
                    WHITESPACE@78..79 " "
                    WORD@79..82 "Raj"
                    WHITESPACE@82..83 " "
                    WORD@83..85 "P."
                    WHITESPACE@85..86 " "
                    WORD@86..89 "and"
                    WHITESPACE@89..90 " "
                    WORD@90..98 "Achuthan"
                    COMMA@98..99 ","
                    WHITESPACE@99..100 " "
                    WORD@100..102 "N."
                    WHITESPACE@102..103 " "
                    WORD@103..105 "R."
                    R_CURLY@105..106 "}"
                  COMMA@106..107 ","
                WHITESPACE@107..112 "\n    "
                FIELD@112..194
                  NAME@112..117 "title"
                  WHITESPACE@117..118 " "
                  EQ@118..119 "="
                  WHITESPACE@119..120 " "
                  CURLY_GROUP@120..193
                    L_CURLY@120..121 "{"
                    WORD@121..122 "A"
                    WHITESPACE@122..123 " "
                    WORD@123..132 "Bottom-up"
                    WHITESPACE@132..133 " "
                    WORD@133..143 "Projection"
                    WHITESPACE@143..144 " "
                    WORD@144..149 "Based"
                    WHITESPACE@149..150 " "
                    WORD@150..159 "Algorithm"
                    WHITESPACE@159..160 " "
                    WORD@160..163 "for"
                    WHITESPACE@163..164 " "
                    WORD@164..170 "Mining"
                    WHITESPACE@170..171 " "
                    WORD@171..175 "High"
                    WHITESPACE@175..176 " "
                    WORD@176..183 "Utility"
                    WHITESPACE@183..184 " "
                    WORD@184..192 "Itemsets"
                    R_CURLY@192..193 "}"
                  COMMA@193..194 ","
                WHITESPACE@194..199 "\n    "
                FIELD@199..213
                  NAME@199..203 "year"
                  WHITESPACE@203..204 " "
                  EQ@204..205 "="
                  WHITESPACE@205..206 " "
                  CURLY_GROUP@206..212
                    L_CURLY@206..207 "{"
                    INTEGER@207..211 "2007"
                    R_CURLY@211..212 "}"
                  COMMA@212..213 ","
                WHITESPACE@213..218 "\n    "
                FIELD@218..241
                  NAME@218..222 "isbn"
                  WHITESPACE@222..223 " "
                  EQ@223..224 "="
                  WHITESPACE@224..225 " "
                  CURLY_GROUP@225..240
                    L_CURLY@225..226 "{"
                    INTEGER@226..239 "9781920682651"
                    R_CURLY@239..240 "}"
                  COMMA@240..241 ","
                WHITESPACE@241..246 "\n    "
                FIELD@246..294
                  NAME@246..255 "publisher"
                  WHITESPACE@255..256 " "
                  EQ@256..257 "="
                  WHITESPACE@257..258 " "
                  CURLY_GROUP@258..293
                    L_CURLY@258..259 "{"
                    WORD@259..269 "Australian"
                    WHITESPACE@269..270 " "
                    WORD@270..278 "Computer"
                    WHITESPACE@278..279 " "
                    WORD@279..286 "Society"
                    COMMA@286..287 ","
                    WHITESPACE@287..288 " "
                    WORD@288..292 "Inc."
                    R_CURLY@292..293 "}"
                  COMMA@293..294 ","
                WHITESPACE@294..299 "\n    "
                FIELD@299..315
                  NAME@299..306 "address"
                  WHITESPACE@306..307 " "
                  EQ@307..308 "="
                  WHITESPACE@308..309 " "
                  CURLY_GROUP@309..314
                    L_CURLY@309..310 "{"
                    WORD@310..313 "AUS"
                    R_CURLY@313..314 "}"
                  COMMA@314..315 ","
                WHITESPACE@315..320 "\n    "
                FIELD@320..447
                  NAME@320..329 "booktitle"
                  WHITESPACE@329..330 " "
                  EQ@330..331 "="
                  WHITESPACE@331..332 " "
                  CURLY_GROUP@332..446
                    L_CURLY@332..333 "{"
                    WORD@333..344 "Proceedings"
                    WHITESPACE@344..345 " "
                    WORD@345..347 "of"
                    WHITESPACE@347..348 " "
                    WORD@348..351 "the"
                    WHITESPACE@351..352 " "
                    WORD@352..355 "2nd"
                    WHITESPACE@355..356 " "
                    WORD@356..369 "International"
                    WHITESPACE@369..370 " "
                    WORD@370..378 "Workshop"
                    WHITESPACE@378..379 " "
                    WORD@379..381 "on"
                    WHITESPACE@381..382 " "
                    WORD@382..393 "Integrating"
                    WHITESPACE@393..394 " "
                    WORD@394..404 "Artificial"
                    WHITESPACE@404..405 " "
                    WORD@405..417 "Intelligence"
                    WHITESPACE@417..418 " "
                    WORD@418..421 "and"
                    WHITESPACE@421..422 " "
                    WORD@422..426 "Data"
                    WHITESPACE@426..427 " "
                    WORD@427..433 "Mining"
                    WHITESPACE@433..434 " "
                    WORD@434..435 "-"
                    WHITESPACE@435..436 " "
                    WORD@436..442 "Volume"
                    WHITESPACE@442..443 " "
                    INTEGER@443..445 "84"
                    R_CURLY@445..446 "}"
                  COMMA@446..447 ","
                WHITESPACE@447..452 "\n    "
                FIELD@452..467
                  NAME@452..457 "pages"
                  WHITESPACE@457..458 " "
                  EQ@458..459 "="
                  WHITESPACE@459..460 " "
                  CURLY_GROUP@460..466
                    L_CURLY@460..461 "{"
                    WORD@461..465 "3-11"
                    R_CURLY@465..466 "}"
                  COMMA@466..467 ","
                WHITESPACE@467..472 "\n    "
                FIELD@472..487
                  NAME@472..480 "numpages"
                  WHITESPACE@480..481 " "
                  EQ@481..482 "="
                  WHITESPACE@482..483 " "
                  CURLY_GROUP@483..486
                    L_CURLY@483..484 "{"
                    INTEGER@484..485 "9"
                    R_CURLY@485..486 "}"
                  COMMA@486..487 ","
                WHITESPACE@487..492 "\n    "
                FIELD@492..549
                  NAME@492..500 "keywords"
                  WHITESPACE@500..501 " "
                  EQ@501..502 "="
                  WHITESPACE@502..503 " "
                  CURLY_GROUP@503..548
                    L_CURLY@503..504 "{"
                    WORD@504..511 "pattern"
                    WHITESPACE@511..512 " "
                    WORD@512..518 "growth"
                    COMMA@518..519 ","
                    WHITESPACE@519..520 " "
                    WORD@520..524 "high"
                    WHITESPACE@524..525 " "
                    WORD@525..532 "utility"
                    WHITESPACE@532..533 " "
                    WORD@533..540 "itemset"
                    WHITESPACE@540..541 " "
                    WORD@541..547 "mining"
                    R_CURLY@547..548 "}"
                  COMMA@548..549 ","
                WHITESPACE@549..554 "\n    "
                FIELD@554..589
                  NAME@554..562 "location"
                  WHITESPACE@562..563 " "
                  EQ@563..564 "="
                  WHITESPACE@564..565 " "
                  CURLY_GROUP@565..588
                    L_CURLY@565..566 "{"
                    WORD@566..570 "Gold"
                    WHITESPACE@570..571 " "
                    WORD@571..576 "Coast"
                    COMMA@576..577 ","
                    WHITESPACE@577..578 " "
                    WORD@578..587 "Australia"
                    R_CURLY@587..588 "}"
                  COMMA@588..589 ","
                WHITESPACE@589..594 "\n    "
                FIELD@594..614
                  NAME@594..600 "series"
                  WHITESPACE@600..601 " "
                  EQ@601..602 "="
                  WHITESPACE@602..603 " "
                  CURLY_GROUP@603..613
                    L_CURLY@603..604 "{"
                    WORD@604..608 "AIDM"
                    WHITESPACE@608..609 " "
                    WORD@609..612 "'07"
                    R_CURLY@612..613 "}"
                  WHITESPACE@613..614 "\n"
                R_DELIM@614..615 "}"

        "#]],
    );
}

#[test]
fn test_jain_1999() {
    check(
        r#"@article{10.1145/331499.331504,
    author = {Jain, A. K. and Murty, M. N. and Flynn, P. J.},
    title = {Data Clustering: A Review},
    year = {1999},
    issue_date = {Sept. 1999},
    publisher = {Association for Computing Machinery},
    address = {New York, NY, USA},
    volume = {31},
    number = {3},
    issn = {0360-0300},
    url = {https://doi.org/10.1145/331499.331504},
    doi = {10.1145/331499.331504},
    journal = {ACM Comput. Surv.},
    month = {sep},
    pages = {264-323},
    numpages = {60},
    keywords = {incremental clustering, clustering applications, exploratory data analysis, cluster analysis, similarity indices, unsupervised learning}
}"#,
        expect![[r#"
            ROOT@0..674
              ENTRY@0..674
                TYPE@0..8 "@article"
                L_DELIM@8..9 "{"
                NAME@9..30 "10.1145/331499.331504"
                COMMA@30..31 ","
                WHITESPACE@31..36 "\n    "
                FIELD@36..93
                  NAME@36..42 "author"
                  WHITESPACE@42..43 " "
                  EQ@43..44 "="
                  WHITESPACE@44..45 " "
                  CURLY_GROUP@45..92
                    L_CURLY@45..46 "{"
                    WORD@46..50 "Jain"
                    COMMA@50..51 ","
                    WHITESPACE@51..52 " "
                    WORD@52..54 "A."
                    WHITESPACE@54..55 " "
                    WORD@55..57 "K."
                    WHITESPACE@57..58 " "
                    WORD@58..61 "and"
                    WHITESPACE@61..62 " "
                    WORD@62..67 "Murty"
                    COMMA@67..68 ","
                    WHITESPACE@68..69 " "
                    WORD@69..71 "M."
                    WHITESPACE@71..72 " "
                    WORD@72..74 "N."
                    WHITESPACE@74..75 " "
                    WORD@75..78 "and"
                    WHITESPACE@78..79 " "
                    WORD@79..84 "Flynn"
                    COMMA@84..85 ","
                    WHITESPACE@85..86 " "
                    WORD@86..88 "P."
                    WHITESPACE@88..89 " "
                    WORD@89..91 "J."
                    R_CURLY@91..92 "}"
                  COMMA@92..93 ","
                WHITESPACE@93..98 "\n    "
                FIELD@98..134
                  NAME@98..103 "title"
                  WHITESPACE@103..104 " "
                  EQ@104..105 "="
                  WHITESPACE@105..106 " "
                  CURLY_GROUP@106..133
                    L_CURLY@106..107 "{"
                    WORD@107..111 "Data"
                    WHITESPACE@111..112 " "
                    WORD@112..123 "Clustering:"
                    WHITESPACE@123..124 " "
                    WORD@124..125 "A"
                    WHITESPACE@125..126 " "
                    WORD@126..132 "Review"
                    R_CURLY@132..133 "}"
                  COMMA@133..134 ","
                WHITESPACE@134..139 "\n    "
                FIELD@139..153
                  NAME@139..143 "year"
                  WHITESPACE@143..144 " "
                  EQ@144..145 "="
                  WHITESPACE@145..146 " "
                  CURLY_GROUP@146..152
                    L_CURLY@146..147 "{"
                    INTEGER@147..151 "1999"
                    R_CURLY@151..152 "}"
                  COMMA@152..153 ","
                WHITESPACE@153..158 "\n    "
                FIELD@158..184
                  NAME@158..168 "issue_date"
                  WHITESPACE@168..169 " "
                  EQ@169..170 "="
                  WHITESPACE@170..171 " "
                  CURLY_GROUP@171..183
                    L_CURLY@171..172 "{"
                    WORD@172..177 "Sept."
                    WHITESPACE@177..178 " "
                    INTEGER@178..182 "1999"
                    R_CURLY@182..183 "}"
                  COMMA@183..184 ","
                WHITESPACE@184..189 "\n    "
                FIELD@189..239
                  NAME@189..198 "publisher"
                  WHITESPACE@198..199 " "
                  EQ@199..200 "="
                  WHITESPACE@200..201 " "
                  CURLY_GROUP@201..238
                    L_CURLY@201..202 "{"
                    WORD@202..213 "Association"
                    WHITESPACE@213..214 " "
                    WORD@214..217 "for"
                    WHITESPACE@217..218 " "
                    WORD@218..227 "Computing"
                    WHITESPACE@227..228 " "
                    WORD@228..237 "Machinery"
                    R_CURLY@237..238 "}"
                  COMMA@238..239 ","
                WHITESPACE@239..244 "\n    "
                FIELD@244..274
                  NAME@244..251 "address"
                  WHITESPACE@251..252 " "
                  EQ@252..253 "="
                  WHITESPACE@253..254 " "
                  CURLY_GROUP@254..273
                    L_CURLY@254..255 "{"
                    WORD@255..258 "New"
                    WHITESPACE@258..259 " "
                    WORD@259..263 "York"
                    COMMA@263..264 ","
                    WHITESPACE@264..265 " "
                    WORD@265..267 "NY"
                    COMMA@267..268 ","
                    WHITESPACE@268..269 " "
                    WORD@269..272 "USA"
                    R_CURLY@272..273 "}"
                  COMMA@273..274 ","
                WHITESPACE@274..279 "\n    "
                FIELD@279..293
                  NAME@279..285 "volume"
                  WHITESPACE@285..286 " "
                  EQ@286..287 "="
                  WHITESPACE@287..288 " "
                  CURLY_GROUP@288..292
                    L_CURLY@288..289 "{"
                    INTEGER@289..291 "31"
                    R_CURLY@291..292 "}"
                  COMMA@292..293 ","
                WHITESPACE@293..298 "\n    "
                FIELD@298..311
                  NAME@298..304 "number"
                  WHITESPACE@304..305 " "
                  EQ@305..306 "="
                  WHITESPACE@306..307 " "
                  CURLY_GROUP@307..310
                    L_CURLY@307..308 "{"
                    INTEGER@308..309 "3"
                    R_CURLY@309..310 "}"
                  COMMA@310..311 ","
                WHITESPACE@311..316 "\n    "
                FIELD@316..335
                  NAME@316..320 "issn"
                  WHITESPACE@320..321 " "
                  EQ@321..322 "="
                  WHITESPACE@322..323 " "
                  CURLY_GROUP@323..334
                    L_CURLY@323..324 "{"
                    WORD@324..333 "0360-0300"
                    R_CURLY@333..334 "}"
                  COMMA@334..335 ","
                WHITESPACE@335..340 "\n    "
                FIELD@340..386
                  NAME@340..343 "url"
                  WHITESPACE@343..344 " "
                  EQ@344..345 "="
                  WHITESPACE@345..346 " "
                  CURLY_GROUP@346..385
                    L_CURLY@346..347 "{"
                    WORD@347..384 "https://doi.org/10.11 ..."
                    R_CURLY@384..385 "}"
                  COMMA@385..386 ","
                WHITESPACE@386..391 "\n    "
                FIELD@391..421
                  NAME@391..394 "doi"
                  WHITESPACE@394..395 " "
                  EQ@395..396 "="
                  WHITESPACE@396..397 " "
                  CURLY_GROUP@397..420
                    L_CURLY@397..398 "{"
                    WORD@398..419 "10.1145/331499.331504"
                    R_CURLY@419..420 "}"
                  COMMA@420..421 ","
                WHITESPACE@421..426 "\n    "
                FIELD@426..456
                  NAME@426..433 "journal"
                  WHITESPACE@433..434 " "
                  EQ@434..435 "="
                  WHITESPACE@435..436 " "
                  CURLY_GROUP@436..455
                    L_CURLY@436..437 "{"
                    WORD@437..440 "ACM"
                    WHITESPACE@440..441 " "
                    WORD@441..448 "Comput."
                    WHITESPACE@448..449 " "
                    WORD@449..454 "Surv."
                    R_CURLY@454..455 "}"
                  COMMA@455..456 ","
                WHITESPACE@456..461 "\n    "
                FIELD@461..475
                  NAME@461..466 "month"
                  WHITESPACE@466..467 " "
                  EQ@467..468 "="
                  WHITESPACE@468..469 " "
                  CURLY_GROUP@469..474
                    L_CURLY@469..470 "{"
                    WORD@470..473 "sep"
                    R_CURLY@473..474 "}"
                  COMMA@474..475 ","
                WHITESPACE@475..480 "\n    "
                FIELD@480..498
                  NAME@480..485 "pages"
                  WHITESPACE@485..486 " "
                  EQ@486..487 "="
                  WHITESPACE@487..488 " "
                  CURLY_GROUP@488..497
                    L_CURLY@488..489 "{"
                    WORD@489..496 "264-323"
                    R_CURLY@496..497 "}"
                  COMMA@497..498 ","
                WHITESPACE@498..503 "\n    "
                FIELD@503..519
                  NAME@503..511 "numpages"
                  WHITESPACE@511..512 " "
                  EQ@512..513 "="
                  WHITESPACE@513..514 " "
                  CURLY_GROUP@514..518
                    L_CURLY@514..515 "{"
                    INTEGER@515..517 "60"
                    R_CURLY@517..518 "}"
                  COMMA@518..519 ","
                WHITESPACE@519..524 "\n    "
                FIELD@524..673
                  NAME@524..532 "keywords"
                  WHITESPACE@532..533 " "
                  EQ@533..534 "="
                  WHITESPACE@534..535 " "
                  CURLY_GROUP@535..672
                    L_CURLY@535..536 "{"
                    WORD@536..547 "incremental"
                    WHITESPACE@547..548 " "
                    WORD@548..558 "clustering"
                    COMMA@558..559 ","
                    WHITESPACE@559..560 " "
                    WORD@560..570 "clustering"
                    WHITESPACE@570..571 " "
                    WORD@571..583 "applications"
                    COMMA@583..584 ","
                    WHITESPACE@584..585 " "
                    WORD@585..596 "exploratory"
                    WHITESPACE@596..597 " "
                    WORD@597..601 "data"
                    WHITESPACE@601..602 " "
                    WORD@602..610 "analysis"
                    COMMA@610..611 ","
                    WHITESPACE@611..612 " "
                    WORD@612..619 "cluster"
                    WHITESPACE@619..620 " "
                    WORD@620..628 "analysis"
                    COMMA@628..629 ","
                    WHITESPACE@629..630 " "
                    WORD@630..640 "similarity"
                    WHITESPACE@640..641 " "
                    WORD@641..648 "indices"
                    COMMA@648..649 ","
                    WHITESPACE@649..650 " "
                    WORD@650..662 "unsupervised"
                    WHITESPACE@662..663 " "
                    WORD@663..671 "learning"
                    R_CURLY@671..672 "}"
                  WHITESPACE@672..673 "\n"
                R_DELIM@673..674 "}"

        "#]],
    );
}

#[test]
fn test_kastenholz_2006() {
    check(
        r#"@string{jchph   = {J.~Chem. Phys.}}

@article{kastenholz,
    author       = {Kastenholz, M. A. and H{\"u}nenberger, Philippe H.},
    title        = {Computation of methodology\hyphen independent ionic solvation
                    free energies from molecular simulations},
    journaltitle = jchph,
    date         = 2006,
    subtitle     = {{I}. {The} electrostatic potential in molecular liquids},
    volume       = 124,
    eid          = 124106,
    doi          = {10.1063/1.2172593},
    langid       = {english},
    langidopts   = {variant=american},
    indextitle   = {Computation of ionic solvation free energies},
    annotation   = {An \texttt{article} entry with an \texttt{eid} and a
                    \texttt{doi} field. Note that the \textsc{doi} is transformed
                    into a clickable link if \texttt{hyperref} support has been
                    enabled},
}"#,
        expect![[r#"
            ROOT@0..898
              STRING@0..35
                TYPE@0..7 "@string"
                L_DELIM@7..8 "{"
                NAME@8..13 "jchph"
                WHITESPACE@13..16 "   "
                EQ@16..17 "="
                WHITESPACE@17..18 " "
                CURLY_GROUP@18..34
                  L_CURLY@18..19 "{"
                  WORD@19..21 "J."
                  NBSP@21..22 "~"
                  WORD@22..27 "Chem."
                  WHITESPACE@27..28 " "
                  WORD@28..33 "Phys."
                  R_CURLY@33..34 "}"
                R_DELIM@34..35 "}"
              JUNK@35..37 "\n\n"
              ENTRY@37..898
                TYPE@37..45 "@article"
                L_DELIM@45..46 "{"
                NAME@46..56 "kastenholz"
                COMMA@56..57 ","
                WHITESPACE@57..62 "\n    "
                FIELD@62..130
                  NAME@62..68 "author"
                  WHITESPACE@68..75 "       "
                  EQ@75..76 "="
                  WHITESPACE@76..77 " "
                  CURLY_GROUP@77..129
                    L_CURLY@77..78 "{"
                    WORD@78..88 "Kastenholz"
                    COMMA@88..89 ","
                    WHITESPACE@89..90 " "
                    WORD@90..92 "M."
                    WHITESPACE@92..93 " "
                    WORD@93..95 "A."
                    WHITESPACE@95..96 " "
                    WORD@96..99 "and"
                    WHITESPACE@99..100 " "
                    WORD@100..101 "H"
                    CURLY_GROUP@101..106
                      L_CURLY@101..102 "{"
                      ACCENT@102..105
                        ACCENT_NAME@102..104 "\\\""
                        WORD@104..105 "u"
                      R_CURLY@105..106 "}"
                    WORD@106..115 "nenberger"
                    COMMA@115..116 ","
                    WHITESPACE@116..117 " "
                    WORD@117..125 "Philippe"
                    WHITESPACE@125..126 " "
                    WORD@126..128 "H."
                    R_CURLY@128..129 "}"
                  COMMA@129..130 ","
                WHITESPACE@130..135 "\n    "
                FIELD@135..275
                  NAME@135..140 "title"
                  WHITESPACE@140..148 "        "
                  EQ@148..149 "="
                  WHITESPACE@149..150 " "
                  CURLY_GROUP@150..274
                    L_CURLY@150..151 "{"
                    WORD@151..162 "Computation"
                    WHITESPACE@162..163 " "
                    WORD@163..165 "of"
                    WHITESPACE@165..166 " "
                    WORD@166..177 "methodology"
                    COMMAND@177..184
                      COMMAND_NAME@177..184 "\\hyphen"
                    WHITESPACE@184..185 " "
                    WORD@185..196 "independent"
                    WHITESPACE@196..197 " "
                    WORD@197..202 "ionic"
                    WHITESPACE@202..203 " "
                    WORD@203..212 "solvation"
                    WHITESPACE@212..233 "\n                    "
                    WORD@233..237 "free"
                    WHITESPACE@237..238 " "
                    WORD@238..246 "energies"
                    WHITESPACE@246..247 " "
                    WORD@247..251 "from"
                    WHITESPACE@251..252 " "
                    WORD@252..261 "molecular"
                    WHITESPACE@261..262 " "
                    WORD@262..273 "simulations"
                    R_CURLY@273..274 "}"
                  COMMA@274..275 ","
                WHITESPACE@275..280 "\n    "
                FIELD@280..301
                  NAME@280..292 "journaltitle"
                  WHITESPACE@292..293 " "
                  EQ@293..294 "="
                  WHITESPACE@294..295 " "
                  LITERAL@295..300
                    NAME@295..300 "jchph"
                  COMMA@300..301 ","
                WHITESPACE@301..306 "\n    "
                FIELD@306..326
                  NAME@306..310 "date"
                  WHITESPACE@310..319 "         "
                  EQ@319..320 "="
                  WHITESPACE@320..321 " "
                  LITERAL@321..325
                    INTEGER@321..325 "2006"
                  COMMA@325..326 ","
                WHITESPACE@326..331 "\n    "
                FIELD@331..404
                  NAME@331..339 "subtitle"
                  WHITESPACE@339..344 "     "
                  EQ@344..345 "="
                  WHITESPACE@345..346 " "
                  CURLY_GROUP@346..403
                    L_CURLY@346..347 "{"
                    CURLY_GROUP@347..350
                      L_CURLY@347..348 "{"
                      WORD@348..349 "I"
                      R_CURLY@349..350 "}"
                    WORD@350..351 "."
                    WHITESPACE@351..352 " "
                    CURLY_GROUP@352..357
                      L_CURLY@352..353 "{"
                      WORD@353..356 "The"
                      R_CURLY@356..357 "}"
                    WHITESPACE@357..358 " "
                    WORD@358..371 "electrostatic"
                    WHITESPACE@371..372 " "
                    WORD@372..381 "potential"
                    WHITESPACE@381..382 " "
                    WORD@382..384 "in"
                    WHITESPACE@384..385 " "
                    WORD@385..394 "molecular"
                    WHITESPACE@394..395 " "
                    WORD@395..402 "liquids"
                    R_CURLY@402..403 "}"
                  COMMA@403..404 ","
                WHITESPACE@404..409 "\n    "
                FIELD@409..428
                  NAME@409..415 "volume"
                  WHITESPACE@415..422 "       "
                  EQ@422..423 "="
                  WHITESPACE@423..424 " "
                  LITERAL@424..427
                    INTEGER@424..427 "124"
                  COMMA@427..428 ","
                WHITESPACE@428..433 "\n    "
                FIELD@433..455
                  NAME@433..436 "eid"
                  WHITESPACE@436..446 "          "
                  EQ@446..447 "="
                  WHITESPACE@447..448 " "
                  LITERAL@448..454
                    INTEGER@448..454 "124106"
                  COMMA@454..455 ","
                WHITESPACE@455..460 "\n    "
                FIELD@460..495
                  NAME@460..463 "doi"
                  WHITESPACE@463..473 "          "
                  EQ@473..474 "="
                  WHITESPACE@474..475 " "
                  CURLY_GROUP@475..494
                    L_CURLY@475..476 "{"
                    WORD@476..493 "10.1063/1.2172593"
                    R_CURLY@493..494 "}"
                  COMMA@494..495 ","
                WHITESPACE@495..500 "\n    "
                FIELD@500..525
                  NAME@500..506 "langid"
                  WHITESPACE@506..513 "       "
                  EQ@513..514 "="
                  WHITESPACE@514..515 " "
                  CURLY_GROUP@515..524
                    L_CURLY@515..516 "{"
                    WORD@516..523 "english"
                    R_CURLY@523..524 "}"
                  COMMA@524..525 ","
                WHITESPACE@525..530 "\n    "
                FIELD@530..564
                  NAME@530..540 "langidopts"
                  WHITESPACE@540..543 "   "
                  EQ@543..544 "="
                  WHITESPACE@544..545 " "
                  CURLY_GROUP@545..563
                    L_CURLY@545..546 "{"
                    WORD@546..562 "variant=american"
                    R_CURLY@562..563 "}"
                  COMMA@563..564 ","
                WHITESPACE@564..569 "\n    "
                FIELD@569..631
                  NAME@569..579 "indextitle"
                  WHITESPACE@579..582 "   "
                  EQ@582..583 "="
                  WHITESPACE@583..584 " "
                  CURLY_GROUP@584..630
                    L_CURLY@584..585 "{"
                    WORD@585..596 "Computation"
                    WHITESPACE@596..597 " "
                    WORD@597..599 "of"
                    WHITESPACE@599..600 " "
                    WORD@600..605 "ionic"
                    WHITESPACE@605..606 " "
                    WORD@606..615 "solvation"
                    WHITESPACE@615..616 " "
                    WORD@616..620 "free"
                    WHITESPACE@620..621 " "
                    WORD@621..629 "energies"
                    R_CURLY@629..630 "}"
                  COMMA@630..631 ","
                WHITESPACE@631..636 "\n    "
                FIELD@636..896
                  NAME@636..646 "annotation"
                  WHITESPACE@646..649 "   "
                  EQ@649..650 "="
                  WHITESPACE@650..651 " "
                  CURLY_GROUP@651..895
                    L_CURLY@651..652 "{"
                    WORD@652..654 "An"
                    WHITESPACE@654..655 " "
                    COMMAND@655..662
                      COMMAND_NAME@655..662 "\\texttt"
                    CURLY_GROUP@662..671
                      L_CURLY@662..663 "{"
                      WORD@663..670 "article"
                      R_CURLY@670..671 "}"
                    WHITESPACE@671..672 " "
                    WORD@672..677 "entry"
                    WHITESPACE@677..678 " "
                    WORD@678..682 "with"
                    WHITESPACE@682..683 " "
                    WORD@683..685 "an"
                    WHITESPACE@685..686 " "
                    COMMAND@686..693
                      COMMAND_NAME@686..693 "\\texttt"
                    CURLY_GROUP@693..698
                      L_CURLY@693..694 "{"
                      WORD@694..697 "eid"
                      R_CURLY@697..698 "}"
                    WHITESPACE@698..699 " "
                    WORD@699..702 "and"
                    WHITESPACE@702..703 " "
                    WORD@703..704 "a"
                    WHITESPACE@704..725 "\n                    "
                    COMMAND@725..732
                      COMMAND_NAME@725..732 "\\texttt"
                    CURLY_GROUP@732..737
                      L_CURLY@732..733 "{"
                      WORD@733..736 "doi"
                      R_CURLY@736..737 "}"
                    WHITESPACE@737..738 " "
                    WORD@738..744 "field."
                    WHITESPACE@744..745 " "
                    WORD@745..749 "Note"
                    WHITESPACE@749..750 " "
                    WORD@750..754 "that"
                    WHITESPACE@754..755 " "
                    WORD@755..758 "the"
                    WHITESPACE@758..759 " "
                    COMMAND@759..766
                      COMMAND_NAME@759..766 "\\textsc"
                    CURLY_GROUP@766..771
                      L_CURLY@766..767 "{"
                      WORD@767..770 "doi"
                      R_CURLY@770..771 "}"
                    WHITESPACE@771..772 " "
                    WORD@772..774 "is"
                    WHITESPACE@774..775 " "
                    WORD@775..786 "transformed"
                    WHITESPACE@786..807 "\n                    "
                    WORD@807..811 "into"
                    WHITESPACE@811..812 " "
                    WORD@812..813 "a"
                    WHITESPACE@813..814 " "
                    WORD@814..823 "clickable"
                    WHITESPACE@823..824 " "
                    WORD@824..828 "link"
                    WHITESPACE@828..829 " "
                    WORD@829..831 "if"
                    WHITESPACE@831..832 " "
                    COMMAND@832..839
                      COMMAND_NAME@832..839 "\\texttt"
                    CURLY_GROUP@839..849
                      L_CURLY@839..840 "{"
                      WORD@840..848 "hyperref"
                      R_CURLY@848..849 "}"
                    WHITESPACE@849..850 " "
                    WORD@850..857 "support"
                    WHITESPACE@857..858 " "
                    WORD@858..861 "has"
                    WHITESPACE@861..862 " "
                    WORD@862..866 "been"
                    WHITESPACE@866..887 "\n                    "
                    WORD@887..894 "enabled"
                    R_CURLY@894..895 "}"
                  COMMA@895..896 ","
                WHITESPACE@896..897 "\n"
                R_DELIM@897..898 "}"

        "#]],
    );
}

#[test]
fn test_knuth_1984() {
    check(
        r#"@book{knuth:ct:a,
    author       = {Knuth, Donald E.},
    title        = {The {\TeX book}},
    date         = 1984,
    maintitle    = {Computers \& Typesetting},
    volume       = {A},
    publisher    = {Addison-Wesley},
    location     = {Reading, Mass.},
    langid       = {english},
    langidopts   = {variant=american},
    sorttitle    = {Computers & Typesetting A},
    indexsorttitle= {The TeXbook},
    indextitle   = {\protect\TeX book, The},
    shorttitle   = {\TeX book},
    annotation   = {The first volume of a five-volume book. Note the
                    \texttt{sorttitle} field. We want this
                    volume to be listed after the entry referring to the entire
                    five-volume set. Also note the \texttt{indextitle} and
                    \texttt{indexsorttitle} fields. Indexing packages that don't
                    generate robust index entries require some control sequences
                    to be protected from expansion},
}"#,
        expect![[r#"
            ROOT@0..993
              ENTRY@0..993
                TYPE@0..5 "@book"
                L_DELIM@5..6 "{"
                NAME@6..16 "knuth:ct:a"
                COMMA@16..17 ","
                WHITESPACE@17..22 "\n    "
                FIELD@22..56
                  NAME@22..28 "author"
                  WHITESPACE@28..35 "       "
                  EQ@35..36 "="
                  WHITESPACE@36..37 " "
                  CURLY_GROUP@37..55
                    L_CURLY@37..38 "{"
                    WORD@38..43 "Knuth"
                    COMMA@43..44 ","
                    WHITESPACE@44..45 " "
                    WORD@45..51 "Donald"
                    WHITESPACE@51..52 " "
                    WORD@52..54 "E."
                    R_CURLY@54..55 "}"
                  COMMA@55..56 ","
                WHITESPACE@56..61 "\n    "
                FIELD@61..94
                  NAME@61..66 "title"
                  WHITESPACE@66..74 "        "
                  EQ@74..75 "="
                  WHITESPACE@75..76 " "
                  CURLY_GROUP@76..93
                    L_CURLY@76..77 "{"
                    WORD@77..80 "The"
                    WHITESPACE@80..81 " "
                    CURLY_GROUP@81..92
                      L_CURLY@81..82 "{"
                      COMMAND@82..86
                        COMMAND_NAME@82..86 "\\TeX"
                      WHITESPACE@86..87 " "
                      WORD@87..91 "book"
                      R_CURLY@91..92 "}"
                    R_CURLY@92..93 "}"
                  COMMA@93..94 ","
                WHITESPACE@94..99 "\n    "
                FIELD@99..119
                  NAME@99..103 "date"
                  WHITESPACE@103..112 "         "
                  EQ@112..113 "="
                  WHITESPACE@113..114 " "
                  LITERAL@114..118
                    INTEGER@114..118 "1984"
                  COMMA@118..119 ","
                WHITESPACE@119..124 "\n    "
                FIELD@124..166
                  NAME@124..133 "maintitle"
                  WHITESPACE@133..137 "    "
                  EQ@137..138 "="
                  WHITESPACE@138..139 " "
                  CURLY_GROUP@139..165
                    L_CURLY@139..140 "{"
                    WORD@140..149 "Computers"
                    WHITESPACE@149..150 " "
                    COMMAND@150..152
                      COMMAND_NAME@150..152 "\\&"
                    WHITESPACE@152..153 " "
                    WORD@153..164 "Typesetting"
                    R_CURLY@164..165 "}"
                  COMMA@165..166 ","
                WHITESPACE@166..171 "\n    "
                FIELD@171..190
                  NAME@171..177 "volume"
                  WHITESPACE@177..184 "       "
                  EQ@184..185 "="
                  WHITESPACE@185..186 " "
                  CURLY_GROUP@186..189
                    L_CURLY@186..187 "{"
                    WORD@187..188 "A"
                    R_CURLY@188..189 "}"
                  COMMA@189..190 ","
                WHITESPACE@190..195 "\n    "
                FIELD@195..227
                  NAME@195..204 "publisher"
                  WHITESPACE@204..208 "    "
                  EQ@208..209 "="
                  WHITESPACE@209..210 " "
                  CURLY_GROUP@210..226
                    L_CURLY@210..211 "{"
                    WORD@211..225 "Addison-Wesley"
                    R_CURLY@225..226 "}"
                  COMMA@226..227 ","
                WHITESPACE@227..232 "\n    "
                FIELD@232..264
                  NAME@232..240 "location"
                  WHITESPACE@240..245 "     "
                  EQ@245..246 "="
                  WHITESPACE@246..247 " "
                  CURLY_GROUP@247..263
                    L_CURLY@247..248 "{"
                    WORD@248..255 "Reading"
                    COMMA@255..256 ","
                    WHITESPACE@256..257 " "
                    WORD@257..262 "Mass."
                    R_CURLY@262..263 "}"
                  COMMA@263..264 ","
                WHITESPACE@264..269 "\n    "
                FIELD@269..294
                  NAME@269..275 "langid"
                  WHITESPACE@275..282 "       "
                  EQ@282..283 "="
                  WHITESPACE@283..284 " "
                  CURLY_GROUP@284..293
                    L_CURLY@284..285 "{"
                    WORD@285..292 "english"
                    R_CURLY@292..293 "}"
                  COMMA@293..294 ","
                WHITESPACE@294..299 "\n    "
                FIELD@299..333
                  NAME@299..309 "langidopts"
                  WHITESPACE@309..312 "   "
                  EQ@312..313 "="
                  WHITESPACE@313..314 " "
                  CURLY_GROUP@314..332
                    L_CURLY@314..315 "{"
                    WORD@315..331 "variant=american"
                    R_CURLY@331..332 "}"
                  COMMA@332..333 ","
                WHITESPACE@333..338 "\n    "
                FIELD@338..381
                  NAME@338..347 "sorttitle"
                  WHITESPACE@347..351 "    "
                  EQ@351..352 "="
                  WHITESPACE@352..353 " "
                  CURLY_GROUP@353..380
                    L_CURLY@353..354 "{"
                    WORD@354..363 "Computers"
                    WHITESPACE@363..364 " "
                    WORD@364..365 "&"
                    WHITESPACE@365..366 " "
                    WORD@366..377 "Typesetting"
                    WHITESPACE@377..378 " "
                    WORD@378..379 "A"
                    R_CURLY@379..380 "}"
                  COMMA@380..381 ","
                WHITESPACE@381..386 "\n    "
                FIELD@386..416
                  NAME@386..400 "indexsorttitle"
                  EQ@400..401 "="
                  WHITESPACE@401..402 " "
                  CURLY_GROUP@402..415
                    L_CURLY@402..403 "{"
                    WORD@403..406 "The"
                    WHITESPACE@406..407 " "
                    WORD@407..414 "TeXbook"
                    R_CURLY@414..415 "}"
                  COMMA@415..416 ","
                WHITESPACE@416..421 "\n    "
                FIELD@421..461
                  NAME@421..431 "indextitle"
                  WHITESPACE@431..434 "   "
                  EQ@434..435 "="
                  WHITESPACE@435..436 " "
                  CURLY_GROUP@436..460
                    L_CURLY@436..437 "{"
                    COMMAND@437..445
                      COMMAND_NAME@437..445 "\\protect"
                    COMMAND@445..449
                      COMMAND_NAME@445..449 "\\TeX"
                    WHITESPACE@449..450 " "
                    WORD@450..454 "book"
                    COMMA@454..455 ","
                    WHITESPACE@455..456 " "
                    WORD@456..459 "The"
                    R_CURLY@459..460 "}"
                  COMMA@460..461 ","
                WHITESPACE@461..466 "\n    "
                FIELD@466..493
                  NAME@466..476 "shorttitle"
                  WHITESPACE@476..479 "   "
                  EQ@479..480 "="
                  WHITESPACE@480..481 " "
                  CURLY_GROUP@481..492
                    L_CURLY@481..482 "{"
                    COMMAND@482..486
                      COMMAND_NAME@482..486 "\\TeX"
                    WHITESPACE@486..487 " "
                    WORD@487..491 "book"
                    R_CURLY@491..492 "}"
                  COMMA@492..493 ","
                WHITESPACE@493..498 "\n    "
                FIELD@498..991
                  NAME@498..508 "annotation"
                  WHITESPACE@508..511 "   "
                  EQ@511..512 "="
                  WHITESPACE@512..513 " "
                  CURLY_GROUP@513..990
                    L_CURLY@513..514 "{"
                    WORD@514..517 "The"
                    WHITESPACE@517..518 " "
                    WORD@518..523 "first"
                    WHITESPACE@523..524 " "
                    WORD@524..530 "volume"
                    WHITESPACE@530..531 " "
                    WORD@531..533 "of"
                    WHITESPACE@533..534 " "
                    WORD@534..535 "a"
                    WHITESPACE@535..536 " "
                    WORD@536..547 "five-volume"
                    WHITESPACE@547..548 " "
                    WORD@548..553 "book."
                    WHITESPACE@553..554 " "
                    WORD@554..558 "Note"
                    WHITESPACE@558..559 " "
                    WORD@559..562 "the"
                    WHITESPACE@562..583 "\n                    "
                    COMMAND@583..590
                      COMMAND_NAME@583..590 "\\texttt"
                    CURLY_GROUP@590..601
                      L_CURLY@590..591 "{"
                      WORD@591..600 "sorttitle"
                      R_CURLY@600..601 "}"
                    WHITESPACE@601..602 " "
                    WORD@602..608 "field."
                    WHITESPACE@608..609 " "
                    WORD@609..611 "We"
                    WHITESPACE@611..612 " "
                    WORD@612..616 "want"
                    WHITESPACE@616..617 " "
                    WORD@617..621 "this"
                    WHITESPACE@621..642 "\n                    "
                    WORD@642..648 "volume"
                    WHITESPACE@648..649 " "
                    WORD@649..651 "to"
                    WHITESPACE@651..652 " "
                    WORD@652..654 "be"
                    WHITESPACE@654..655 " "
                    WORD@655..661 "listed"
                    WHITESPACE@661..662 " "
                    WORD@662..667 "after"
                    WHITESPACE@667..668 " "
                    WORD@668..671 "the"
                    WHITESPACE@671..672 " "
                    WORD@672..677 "entry"
                    WHITESPACE@677..678 " "
                    WORD@678..687 "referring"
                    WHITESPACE@687..688 " "
                    WORD@688..690 "to"
                    WHITESPACE@690..691 " "
                    WORD@691..694 "the"
                    WHITESPACE@694..695 " "
                    WORD@695..701 "entire"
                    WHITESPACE@701..722 "\n                    "
                    WORD@722..733 "five-volume"
                    WHITESPACE@733..734 " "
                    WORD@734..738 "set."
                    WHITESPACE@738..739 " "
                    WORD@739..743 "Also"
                    WHITESPACE@743..744 " "
                    WORD@744..748 "note"
                    WHITESPACE@748..749 " "
                    WORD@749..752 "the"
                    WHITESPACE@752..753 " "
                    COMMAND@753..760
                      COMMAND_NAME@753..760 "\\texttt"
                    CURLY_GROUP@760..772
                      L_CURLY@760..761 "{"
                      WORD@761..771 "indextitle"
                      R_CURLY@771..772 "}"
                    WHITESPACE@772..773 " "
                    WORD@773..776 "and"
                    WHITESPACE@776..797 "\n                    "
                    COMMAND@797..804
                      COMMAND_NAME@797..804 "\\texttt"
                    CURLY_GROUP@804..820
                      L_CURLY@804..805 "{"
                      WORD@805..819 "indexsorttitle"
                      R_CURLY@819..820 "}"
                    WHITESPACE@820..821 " "
                    WORD@821..828 "fields."
                    WHITESPACE@828..829 " "
                    WORD@829..837 "Indexing"
                    WHITESPACE@837..838 " "
                    WORD@838..846 "packages"
                    WHITESPACE@846..847 " "
                    WORD@847..851 "that"
                    WHITESPACE@851..852 " "
                    WORD@852..857 "don't"
                    WHITESPACE@857..878 "\n                    "
                    WORD@878..886 "generate"
                    WHITESPACE@886..887 " "
                    WORD@887..893 "robust"
                    WHITESPACE@893..894 " "
                    WORD@894..899 "index"
                    WHITESPACE@899..900 " "
                    WORD@900..907 "entries"
                    WHITESPACE@907..908 " "
                    WORD@908..915 "require"
                    WHITESPACE@915..916 " "
                    WORD@916..920 "some"
                    WHITESPACE@920..921 " "
                    WORD@921..928 "control"
                    WHITESPACE@928..929 " "
                    WORD@929..938 "sequences"
                    WHITESPACE@938..959 "\n                    "
                    WORD@959..961 "to"
                    WHITESPACE@961..962 " "
                    WORD@962..964 "be"
                    WHITESPACE@964..965 " "
                    WORD@965..974 "protected"
                    WHITESPACE@974..975 " "
                    WORD@975..979 "from"
                    WHITESPACE@979..980 " "
                    WORD@980..989 "expansion"
                    R_CURLY@989..990 "}"
                  COMMA@990..991 ","
                WHITESPACE@991..992 "\n"
                R_DELIM@992..993 "}"

        "#]],
    );
}

#[test]
fn test_matuz_1990() {
    check(
        r#"@collection{matuz:doody,
    editor       = {Matuz, Roger},
    title        = {Contemporary Literary Criticism},
    year         = 1990,
    volume       = 61,
    publisher    = {Gale},
    location     = {Detroit},
    pages        = {204-208},
    langid       = {english},
    langidopts   = {variant=american},
    annotation   = {A \texttt{collection} entry providing the excerpt information
                    for the \texttt{doody} entry. Note the format of the
                    \texttt{pages} field},
}"#,
        expect![[r#"
            ROOT@0..517
              ENTRY@0..517
                TYPE@0..11 "@collection"
                L_DELIM@11..12 "{"
                NAME@12..23 "matuz:doody"
                COMMA@23..24 ","
                WHITESPACE@24..29 "\n    "
                FIELD@29..59
                  NAME@29..35 "editor"
                  WHITESPACE@35..42 "       "
                  EQ@42..43 "="
                  WHITESPACE@43..44 " "
                  CURLY_GROUP@44..58
                    L_CURLY@44..45 "{"
                    WORD@45..50 "Matuz"
                    COMMA@50..51 ","
                    WHITESPACE@51..52 " "
                    WORD@52..57 "Roger"
                    R_CURLY@57..58 "}"
                  COMMA@58..59 ","
                WHITESPACE@59..64 "\n    "
                FIELD@64..113
                  NAME@64..69 "title"
                  WHITESPACE@69..77 "        "
                  EQ@77..78 "="
                  WHITESPACE@78..79 " "
                  CURLY_GROUP@79..112
                    L_CURLY@79..80 "{"
                    WORD@80..92 "Contemporary"
                    WHITESPACE@92..93 " "
                    WORD@93..101 "Literary"
                    WHITESPACE@101..102 " "
                    WORD@102..111 "Criticism"
                    R_CURLY@111..112 "}"
                  COMMA@112..113 ","
                WHITESPACE@113..118 "\n    "
                FIELD@118..138
                  NAME@118..122 "year"
                  WHITESPACE@122..131 "         "
                  EQ@131..132 "="
                  WHITESPACE@132..133 " "
                  LITERAL@133..137
                    INTEGER@133..137 "1990"
                  COMMA@137..138 ","
                WHITESPACE@138..143 "\n    "
                FIELD@143..161
                  NAME@143..149 "volume"
                  WHITESPACE@149..156 "       "
                  EQ@156..157 "="
                  WHITESPACE@157..158 " "
                  LITERAL@158..160
                    INTEGER@158..160 "61"
                  COMMA@160..161 ","
                WHITESPACE@161..166 "\n    "
                FIELD@166..188
                  NAME@166..175 "publisher"
                  WHITESPACE@175..179 "    "
                  EQ@179..180 "="
                  WHITESPACE@180..181 " "
                  CURLY_GROUP@181..187
                    L_CURLY@181..182 "{"
                    WORD@182..186 "Gale"
                    R_CURLY@186..187 "}"
                  COMMA@187..188 ","
                WHITESPACE@188..193 "\n    "
                FIELD@193..218
                  NAME@193..201 "location"
                  WHITESPACE@201..206 "     "
                  EQ@206..207 "="
                  WHITESPACE@207..208 " "
                  CURLY_GROUP@208..217
                    L_CURLY@208..209 "{"
                    WORD@209..216 "Detroit"
                    R_CURLY@216..217 "}"
                  COMMA@217..218 ","
                WHITESPACE@218..223 "\n    "
                FIELD@223..248
                  NAME@223..228 "pages"
                  WHITESPACE@228..236 "        "
                  EQ@236..237 "="
                  WHITESPACE@237..238 " "
                  CURLY_GROUP@238..247
                    L_CURLY@238..239 "{"
                    WORD@239..246 "204-208"
                    R_CURLY@246..247 "}"
                  COMMA@247..248 ","
                WHITESPACE@248..253 "\n    "
                FIELD@253..278
                  NAME@253..259 "langid"
                  WHITESPACE@259..266 "       "
                  EQ@266..267 "="
                  WHITESPACE@267..268 " "
                  CURLY_GROUP@268..277
                    L_CURLY@268..269 "{"
                    WORD@269..276 "english"
                    R_CURLY@276..277 "}"
                  COMMA@277..278 ","
                WHITESPACE@278..283 "\n    "
                FIELD@283..317
                  NAME@283..293 "langidopts"
                  WHITESPACE@293..296 "   "
                  EQ@296..297 "="
                  WHITESPACE@297..298 " "
                  CURLY_GROUP@298..316
                    L_CURLY@298..299 "{"
                    WORD@299..315 "variant=american"
                    R_CURLY@315..316 "}"
                  COMMA@316..317 ","
                WHITESPACE@317..322 "\n    "
                FIELD@322..515
                  NAME@322..332 "annotation"
                  WHITESPACE@332..335 "   "
                  EQ@335..336 "="
                  WHITESPACE@336..337 " "
                  CURLY_GROUP@337..514
                    L_CURLY@337..338 "{"
                    WORD@338..339 "A"
                    WHITESPACE@339..340 " "
                    COMMAND@340..347
                      COMMAND_NAME@340..347 "\\texttt"
                    CURLY_GROUP@347..359
                      L_CURLY@347..348 "{"
                      WORD@348..358 "collection"
                      R_CURLY@358..359 "}"
                    WHITESPACE@359..360 " "
                    WORD@360..365 "entry"
                    WHITESPACE@365..366 " "
                    WORD@366..375 "providing"
                    WHITESPACE@375..376 " "
                    WORD@376..379 "the"
                    WHITESPACE@379..380 " "
                    WORD@380..387 "excerpt"
                    WHITESPACE@387..388 " "
                    WORD@388..399 "information"
                    WHITESPACE@399..420 "\n                    "
                    WORD@420..423 "for"
                    WHITESPACE@423..424 " "
                    WORD@424..427 "the"
                    WHITESPACE@427..428 " "
                    COMMAND@428..435
                      COMMAND_NAME@428..435 "\\texttt"
                    CURLY_GROUP@435..442
                      L_CURLY@435..436 "{"
                      WORD@436..441 "doody"
                      R_CURLY@441..442 "}"
                    WHITESPACE@442..443 " "
                    WORD@443..449 "entry."
                    WHITESPACE@449..450 " "
                    WORD@450..454 "Note"
                    WHITESPACE@454..455 " "
                    WORD@455..458 "the"
                    WHITESPACE@458..459 " "
                    WORD@459..465 "format"
                    WHITESPACE@465..466 " "
                    WORD@466..468 "of"
                    WHITESPACE@468..469 " "
                    WORD@469..472 "the"
                    WHITESPACE@472..493 "\n                    "
                    COMMAND@493..500
                      COMMAND_NAME@493..500 "\\texttt"
                    CURLY_GROUP@500..507
                      L_CURLY@500..501 "{"
                      WORD@501..506 "pages"
                      R_CURLY@506..507 "}"
                    WHITESPACE@507..508 " "
                    WORD@508..513 "field"
                    R_CURLY@513..514 "}"
                  COMMA@514..515 ","
                WHITESPACE@515..516 "\n"
                R_DELIM@516..517 "}"

        "#]],
    );
}

#[test]
fn test_nietzsche_1998() {
    check(
        r#"@string{dtv     = {Deutscher Taschenbuch-Verlag}}

@mvbook{nietzsche:ksa,
    author       = {Nietzsche, Friedrich},
    title        = {S{\"a}mtliche Werke},
    date         = 1988,
    editor       = {Colli, Giorgio and Montinari, Mazzino},
    edition      = 2,
    volumes      = 15,
    publisher    = dtv # { and Walter de Gruyter},
    location     = {M{\"u}nchen and Berlin and New York},
    langid       = {german},
    sorttitle    = {Werke-00-000},
    indexsorttitle= {Samtliche Werke},
    subtitle     = {Kritische Studienausgabe},
    annotation   = {The critical edition of Nietzsche's works. This is a
                    \texttt{mvbook} entry referring to a 15-volume work as a
                    whole. Note the \texttt{volumes} field and the format of the
                    \texttt{publisher} and \texttt{location} fields in the
                    database file. Also note the \texttt{sorttitle} and
                    field which is used to fine-tune the
                    sorting order of the bibliography. We want this item listed
                    first in the bibliography},
}"#,
        expect![[r##"
            ROOT@0..1112
              STRING@0..49
                TYPE@0..7 "@string"
                L_DELIM@7..8 "{"
                NAME@8..11 "dtv"
                WHITESPACE@11..16 "     "
                EQ@16..17 "="
                WHITESPACE@17..18 " "
                CURLY_GROUP@18..48
                  L_CURLY@18..19 "{"
                  WORD@19..28 "Deutscher"
                  WHITESPACE@28..29 " "
                  WORD@29..47 "Taschenbuch-Verlag"
                  R_CURLY@47..48 "}"
                R_DELIM@48..49 "}"
              JUNK@49..51 "\n\n"
              ENTRY@51..1112
                TYPE@51..58 "@mvbook"
                L_DELIM@58..59 "{"
                NAME@59..72 "nietzsche:ksa"
                COMMA@72..73 ","
                WHITESPACE@73..78 "\n    "
                FIELD@78..116
                  NAME@78..84 "author"
                  WHITESPACE@84..91 "       "
                  EQ@91..92 "="
                  WHITESPACE@92..93 " "
                  CURLY_GROUP@93..115
                    L_CURLY@93..94 "{"
                    WORD@94..103 "Nietzsche"
                    COMMA@103..104 ","
                    WHITESPACE@104..105 " "
                    WORD@105..114 "Friedrich"
                    R_CURLY@114..115 "}"
                  COMMA@115..116 ","
                WHITESPACE@116..121 "\n    "
                FIELD@121..158
                  NAME@121..126 "title"
                  WHITESPACE@126..134 "        "
                  EQ@134..135 "="
                  WHITESPACE@135..136 " "
                  CURLY_GROUP@136..157
                    L_CURLY@136..137 "{"
                    WORD@137..138 "S"
                    CURLY_GROUP@138..143
                      L_CURLY@138..139 "{"
                      ACCENT@139..142
                        ACCENT_NAME@139..141 "\\\""
                        WORD@141..142 "a"
                      R_CURLY@142..143 "}"
                    WORD@143..150 "mtliche"
                    WHITESPACE@150..151 " "
                    WORD@151..156 "Werke"
                    R_CURLY@156..157 "}"
                  COMMA@157..158 ","
                WHITESPACE@158..163 "\n    "
                FIELD@163..183
                  NAME@163..167 "date"
                  WHITESPACE@167..176 "         "
                  EQ@176..177 "="
                  WHITESPACE@177..178 " "
                  LITERAL@178..182
                    INTEGER@178..182 "1988"
                  COMMA@182..183 ","
                WHITESPACE@183..188 "\n    "
                FIELD@188..243
                  NAME@188..194 "editor"
                  WHITESPACE@194..201 "       "
                  EQ@201..202 "="
                  WHITESPACE@202..203 " "
                  CURLY_GROUP@203..242
                    L_CURLY@203..204 "{"
                    WORD@204..209 "Colli"
                    COMMA@209..210 ","
                    WHITESPACE@210..211 " "
                    WORD@211..218 "Giorgio"
                    WHITESPACE@218..219 " "
                    WORD@219..222 "and"
                    WHITESPACE@222..223 " "
                    WORD@223..232 "Montinari"
                    COMMA@232..233 ","
                    WHITESPACE@233..234 " "
                    WORD@234..241 "Mazzino"
                    R_CURLY@241..242 "}"
                  COMMA@242..243 ","
                WHITESPACE@243..248 "\n    "
                FIELD@248..265
                  NAME@248..255 "edition"
                  WHITESPACE@255..261 "      "
                  EQ@261..262 "="
                  WHITESPACE@262..263 " "
                  LITERAL@263..264
                    INTEGER@263..264 "2"
                  COMMA@264..265 ","
                WHITESPACE@265..270 "\n    "
                FIELD@270..288
                  NAME@270..277 "volumes"
                  WHITESPACE@277..283 "      "
                  EQ@283..284 "="
                  WHITESPACE@284..285 " "
                  LITERAL@285..287
                    INTEGER@285..287 "15"
                  COMMA@287..288 ","
                WHITESPACE@288..293 "\n    "
                FIELD@293..339
                  NAME@293..302 "publisher"
                  WHITESPACE@302..306 "    "
                  EQ@306..307 "="
                  WHITESPACE@307..308 " "
                  JOIN@308..338
                    LITERAL@308..311
                      NAME@308..311 "dtv"
                    WHITESPACE@311..312 " "
                    POUND@312..313 "#"
                    WHITESPACE@313..314 " "
                    CURLY_GROUP@314..338
                      L_CURLY@314..315 "{"
                      WHITESPACE@315..316 " "
                      WORD@316..319 "and"
                      WHITESPACE@319..320 " "
                      WORD@320..326 "Walter"
                      WHITESPACE@326..327 " "
                      WORD@327..329 "de"
                      WHITESPACE@329..330 " "
                      WORD@330..337 "Gruyter"
                      R_CURLY@337..338 "}"
                  COMMA@338..339 ","
                WHITESPACE@339..344 "\n    "
                FIELD@344..397
                  NAME@344..352 "location"
                  WHITESPACE@352..357 "     "
                  EQ@357..358 "="
                  WHITESPACE@358..359 " "
                  CURLY_GROUP@359..396
                    L_CURLY@359..360 "{"
                    WORD@360..361 "M"
                    CURLY_GROUP@361..366
                      L_CURLY@361..362 "{"
                      ACCENT@362..365
                        ACCENT_NAME@362..364 "\\\""
                        WORD@364..365 "u"
                      R_CURLY@365..366 "}"
                    WORD@366..371 "nchen"
                    WHITESPACE@371..372 " "
                    WORD@372..375 "and"
                    WHITESPACE@375..376 " "
                    WORD@376..382 "Berlin"
                    WHITESPACE@382..383 " "
                    WORD@383..386 "and"
                    WHITESPACE@386..387 " "
                    WORD@387..390 "New"
                    WHITESPACE@390..391 " "
                    WORD@391..395 "York"
                    R_CURLY@395..396 "}"
                  COMMA@396..397 ","
                WHITESPACE@397..402 "\n    "
                FIELD@402..426
                  NAME@402..408 "langid"
                  WHITESPACE@408..415 "       "
                  EQ@415..416 "="
                  WHITESPACE@416..417 " "
                  CURLY_GROUP@417..425
                    L_CURLY@417..418 "{"
                    WORD@418..424 "german"
                    R_CURLY@424..425 "}"
                  COMMA@425..426 ","
                WHITESPACE@426..431 "\n    "
                FIELD@431..461
                  NAME@431..440 "sorttitle"
                  WHITESPACE@440..444 "    "
                  EQ@444..445 "="
                  WHITESPACE@445..446 " "
                  CURLY_GROUP@446..460
                    L_CURLY@446..447 "{"
                    WORD@447..459 "Werke-00-000"
                    R_CURLY@459..460 "}"
                  COMMA@460..461 ","
                WHITESPACE@461..466 "\n    "
                FIELD@466..500
                  NAME@466..480 "indexsorttitle"
                  EQ@480..481 "="
                  WHITESPACE@481..482 " "
                  CURLY_GROUP@482..499
                    L_CURLY@482..483 "{"
                    WORD@483..492 "Samtliche"
                    WHITESPACE@492..493 " "
                    WORD@493..498 "Werke"
                    R_CURLY@498..499 "}"
                  COMMA@499..500 ","
                WHITESPACE@500..505 "\n    "
                FIELD@505..547
                  NAME@505..513 "subtitle"
                  WHITESPACE@513..518 "     "
                  EQ@518..519 "="
                  WHITESPACE@519..520 " "
                  CURLY_GROUP@520..546
                    L_CURLY@520..521 "{"
                    WORD@521..530 "Kritische"
                    WHITESPACE@530..531 " "
                    WORD@531..545 "Studienausgabe"
                    R_CURLY@545..546 "}"
                  COMMA@546..547 ","
                WHITESPACE@547..552 "\n    "
                FIELD@552..1110
                  NAME@552..562 "annotation"
                  WHITESPACE@562..565 "   "
                  EQ@565..566 "="
                  WHITESPACE@566..567 " "
                  CURLY_GROUP@567..1109
                    L_CURLY@567..568 "{"
                    WORD@568..571 "The"
                    WHITESPACE@571..572 " "
                    WORD@572..580 "critical"
                    WHITESPACE@580..581 " "
                    WORD@581..588 "edition"
                    WHITESPACE@588..589 " "
                    WORD@589..591 "of"
                    WHITESPACE@591..592 " "
                    WORD@592..603 "Nietzsche's"
                    WHITESPACE@603..604 " "
                    WORD@604..610 "works."
                    WHITESPACE@610..611 " "
                    WORD@611..615 "This"
                    WHITESPACE@615..616 " "
                    WORD@616..618 "is"
                    WHITESPACE@618..619 " "
                    WORD@619..620 "a"
                    WHITESPACE@620..641 "\n                    "
                    COMMAND@641..648
                      COMMAND_NAME@641..648 "\\texttt"
                    CURLY_GROUP@648..656
                      L_CURLY@648..649 "{"
                      WORD@649..655 "mvbook"
                      R_CURLY@655..656 "}"
                    WHITESPACE@656..657 " "
                    WORD@657..662 "entry"
                    WHITESPACE@662..663 " "
                    WORD@663..672 "referring"
                    WHITESPACE@672..673 " "
                    WORD@673..675 "to"
                    WHITESPACE@675..676 " "
                    WORD@676..677 "a"
                    WHITESPACE@677..678 " "
                    WORD@678..687 "15-volume"
                    WHITESPACE@687..688 " "
                    WORD@688..692 "work"
                    WHITESPACE@692..693 " "
                    WORD@693..695 "as"
                    WHITESPACE@695..696 " "
                    WORD@696..697 "a"
                    WHITESPACE@697..718 "\n                    "
                    WORD@718..724 "whole."
                    WHITESPACE@724..725 " "
                    WORD@725..729 "Note"
                    WHITESPACE@729..730 " "
                    WORD@730..733 "the"
                    WHITESPACE@733..734 " "
                    COMMAND@734..741
                      COMMAND_NAME@734..741 "\\texttt"
                    CURLY_GROUP@741..750
                      L_CURLY@741..742 "{"
                      WORD@742..749 "volumes"
                      R_CURLY@749..750 "}"
                    WHITESPACE@750..751 " "
                    WORD@751..756 "field"
                    WHITESPACE@756..757 " "
                    WORD@757..760 "and"
                    WHITESPACE@760..761 " "
                    WORD@761..764 "the"
                    WHITESPACE@764..765 " "
                    WORD@765..771 "format"
                    WHITESPACE@771..772 " "
                    WORD@772..774 "of"
                    WHITESPACE@774..775 " "
                    WORD@775..778 "the"
                    WHITESPACE@778..799 "\n                    "
                    COMMAND@799..806
                      COMMAND_NAME@799..806 "\\texttt"
                    CURLY_GROUP@806..817
                      L_CURLY@806..807 "{"
                      WORD@807..816 "publisher"
                      R_CURLY@816..817 "}"
                    WHITESPACE@817..818 " "
                    WORD@818..821 "and"
                    WHITESPACE@821..822 " "
                    COMMAND@822..829
                      COMMAND_NAME@822..829 "\\texttt"
                    CURLY_GROUP@829..839
                      L_CURLY@829..830 "{"
                      WORD@830..838 "location"
                      R_CURLY@838..839 "}"
                    WHITESPACE@839..840 " "
                    WORD@840..846 "fields"
                    WHITESPACE@846..847 " "
                    WORD@847..849 "in"
                    WHITESPACE@849..850 " "
                    WORD@850..853 "the"
                    WHITESPACE@853..874 "\n                    "
                    WORD@874..882 "database"
                    WHITESPACE@882..883 " "
                    WORD@883..888 "file."
                    WHITESPACE@888..889 " "
                    WORD@889..893 "Also"
                    WHITESPACE@893..894 " "
                    WORD@894..898 "note"
                    WHITESPACE@898..899 " "
                    WORD@899..902 "the"
                    WHITESPACE@902..903 " "
                    COMMAND@903..910
                      COMMAND_NAME@903..910 "\\texttt"
                    CURLY_GROUP@910..921
                      L_CURLY@910..911 "{"
                      WORD@911..920 "sorttitle"
                      R_CURLY@920..921 "}"
                    WHITESPACE@921..922 " "
                    WORD@922..925 "and"
                    WHITESPACE@925..946 "\n                    "
                    WORD@946..951 "field"
                    WHITESPACE@951..952 " "
                    WORD@952..957 "which"
                    WHITESPACE@957..958 " "
                    WORD@958..960 "is"
                    WHITESPACE@960..961 " "
                    WORD@961..965 "used"
                    WHITESPACE@965..966 " "
                    WORD@966..968 "to"
                    WHITESPACE@968..969 " "
                    WORD@969..978 "fine-tune"
                    WHITESPACE@978..979 " "
                    WORD@979..982 "the"
                    WHITESPACE@982..1003 "\n                    "
                    WORD@1003..1010 "sorting"
                    WHITESPACE@1010..1011 " "
                    WORD@1011..1016 "order"
                    WHITESPACE@1016..1017 " "
                    WORD@1017..1019 "of"
                    WHITESPACE@1019..1020 " "
                    WORD@1020..1023 "the"
                    WHITESPACE@1023..1024 " "
                    WORD@1024..1037 "bibliography."
                    WHITESPACE@1037..1038 " "
                    WORD@1038..1040 "We"
                    WHITESPACE@1040..1041 " "
                    WORD@1041..1045 "want"
                    WHITESPACE@1045..1046 " "
                    WORD@1046..1050 "this"
                    WHITESPACE@1050..1051 " "
                    WORD@1051..1055 "item"
                    WHITESPACE@1055..1056 " "
                    WORD@1056..1062 "listed"
                    WHITESPACE@1062..1083 "\n                    "
                    WORD@1083..1088 "first"
                    WHITESPACE@1088..1089 " "
                    WORD@1089..1091 "in"
                    WHITESPACE@1091..1092 " "
                    WORD@1092..1095 "the"
                    WHITESPACE@1095..1096 " "
                    WORD@1096..1108 "bibliography"
                    R_CURLY@1108..1109 "}"
                  COMMA@1109..1110 ","
                WHITESPACE@1110..1111 "\n"
                R_DELIM@1111..1112 "}"

        "##]],
    );
}

#[test]
fn test_rivest_1978() {
    check(
        r#"@article{10.1145/359340.359342,
    author = {Rivest, R. L. and Shamir, A. and Adleman, L.},
    title = {A Method for Obtaining Digital Signatures and Public-Key Cryptosystems},
    year = {1978},
    issue_date = {Feb. 1978},
    publisher = {Association for Computing Machinery},
    address = {New York, NY, USA},
    volume = {21},
    number = {2},
    issn = {0001-0782},
    url = {https://doi.org/10.1145/359340.359342},
    doi = {10.1145/359340.359342},
    journal = {Commun. ACM},
    month = {feb},
    pages = {120-126},
    numpages = {7},
}"#,
        expect![[r#"
            ROOT@0..557
              ENTRY@0..557
                TYPE@0..8 "@article"
                L_DELIM@8..9 "{"
                NAME@9..30 "10.1145/359340.359342"
                COMMA@30..31 ","
                WHITESPACE@31..36 "\n    "
                FIELD@36..92
                  NAME@36..42 "author"
                  WHITESPACE@42..43 " "
                  EQ@43..44 "="
                  WHITESPACE@44..45 " "
                  CURLY_GROUP@45..91
                    L_CURLY@45..46 "{"
                    WORD@46..52 "Rivest"
                    COMMA@52..53 ","
                    WHITESPACE@53..54 " "
                    WORD@54..56 "R."
                    WHITESPACE@56..57 " "
                    WORD@57..59 "L."
                    WHITESPACE@59..60 " "
                    WORD@60..63 "and"
                    WHITESPACE@63..64 " "
                    WORD@64..70 "Shamir"
                    COMMA@70..71 ","
                    WHITESPACE@71..72 " "
                    WORD@72..74 "A."
                    WHITESPACE@74..75 " "
                    WORD@75..78 "and"
                    WHITESPACE@78..79 " "
                    WORD@79..86 "Adleman"
                    COMMA@86..87 ","
                    WHITESPACE@87..88 " "
                    WORD@88..90 "L."
                    R_CURLY@90..91 "}"
                  COMMA@91..92 ","
                WHITESPACE@92..97 "\n    "
                FIELD@97..178
                  NAME@97..102 "title"
                  WHITESPACE@102..103 " "
                  EQ@103..104 "="
                  WHITESPACE@104..105 " "
                  CURLY_GROUP@105..177
                    L_CURLY@105..106 "{"
                    WORD@106..107 "A"
                    WHITESPACE@107..108 " "
                    WORD@108..114 "Method"
                    WHITESPACE@114..115 " "
                    WORD@115..118 "for"
                    WHITESPACE@118..119 " "
                    WORD@119..128 "Obtaining"
                    WHITESPACE@128..129 " "
                    WORD@129..136 "Digital"
                    WHITESPACE@136..137 " "
                    WORD@137..147 "Signatures"
                    WHITESPACE@147..148 " "
                    WORD@148..151 "and"
                    WHITESPACE@151..152 " "
                    WORD@152..162 "Public-Key"
                    WHITESPACE@162..163 " "
                    WORD@163..176 "Cryptosystems"
                    R_CURLY@176..177 "}"
                  COMMA@177..178 ","
                WHITESPACE@178..183 "\n    "
                FIELD@183..197
                  NAME@183..187 "year"
                  WHITESPACE@187..188 " "
                  EQ@188..189 "="
                  WHITESPACE@189..190 " "
                  CURLY_GROUP@190..196
                    L_CURLY@190..191 "{"
                    INTEGER@191..195 "1978"
                    R_CURLY@195..196 "}"
                  COMMA@196..197 ","
                WHITESPACE@197..202 "\n    "
                FIELD@202..227
                  NAME@202..212 "issue_date"
                  WHITESPACE@212..213 " "
                  EQ@213..214 "="
                  WHITESPACE@214..215 " "
                  CURLY_GROUP@215..226
                    L_CURLY@215..216 "{"
                    WORD@216..220 "Feb."
                    WHITESPACE@220..221 " "
                    INTEGER@221..225 "1978"
                    R_CURLY@225..226 "}"
                  COMMA@226..227 ","
                WHITESPACE@227..232 "\n    "
                FIELD@232..282
                  NAME@232..241 "publisher"
                  WHITESPACE@241..242 " "
                  EQ@242..243 "="
                  WHITESPACE@243..244 " "
                  CURLY_GROUP@244..281
                    L_CURLY@244..245 "{"
                    WORD@245..256 "Association"
                    WHITESPACE@256..257 " "
                    WORD@257..260 "for"
                    WHITESPACE@260..261 " "
                    WORD@261..270 "Computing"
                    WHITESPACE@270..271 " "
                    WORD@271..280 "Machinery"
                    R_CURLY@280..281 "}"
                  COMMA@281..282 ","
                WHITESPACE@282..287 "\n    "
                FIELD@287..317
                  NAME@287..294 "address"
                  WHITESPACE@294..295 " "
                  EQ@295..296 "="
                  WHITESPACE@296..297 " "
                  CURLY_GROUP@297..316
                    L_CURLY@297..298 "{"
                    WORD@298..301 "New"
                    WHITESPACE@301..302 " "
                    WORD@302..306 "York"
                    COMMA@306..307 ","
                    WHITESPACE@307..308 " "
                    WORD@308..310 "NY"
                    COMMA@310..311 ","
                    WHITESPACE@311..312 " "
                    WORD@312..315 "USA"
                    R_CURLY@315..316 "}"
                  COMMA@316..317 ","
                WHITESPACE@317..322 "\n    "
                FIELD@322..336
                  NAME@322..328 "volume"
                  WHITESPACE@328..329 " "
                  EQ@329..330 "="
                  WHITESPACE@330..331 " "
                  CURLY_GROUP@331..335
                    L_CURLY@331..332 "{"
                    INTEGER@332..334 "21"
                    R_CURLY@334..335 "}"
                  COMMA@335..336 ","
                WHITESPACE@336..341 "\n    "
                FIELD@341..354
                  NAME@341..347 "number"
                  WHITESPACE@347..348 " "
                  EQ@348..349 "="
                  WHITESPACE@349..350 " "
                  CURLY_GROUP@350..353
                    L_CURLY@350..351 "{"
                    INTEGER@351..352 "2"
                    R_CURLY@352..353 "}"
                  COMMA@353..354 ","
                WHITESPACE@354..359 "\n    "
                FIELD@359..378
                  NAME@359..363 "issn"
                  WHITESPACE@363..364 " "
                  EQ@364..365 "="
                  WHITESPACE@365..366 " "
                  CURLY_GROUP@366..377
                    L_CURLY@366..367 "{"
                    WORD@367..376 "0001-0782"
                    R_CURLY@376..377 "}"
                  COMMA@377..378 ","
                WHITESPACE@378..383 "\n    "
                FIELD@383..429
                  NAME@383..386 "url"
                  WHITESPACE@386..387 " "
                  EQ@387..388 "="
                  WHITESPACE@388..389 " "
                  CURLY_GROUP@389..428
                    L_CURLY@389..390 "{"
                    WORD@390..427 "https://doi.org/10.11 ..."
                    R_CURLY@427..428 "}"
                  COMMA@428..429 ","
                WHITESPACE@429..434 "\n    "
                FIELD@434..464
                  NAME@434..437 "doi"
                  WHITESPACE@437..438 " "
                  EQ@438..439 "="
                  WHITESPACE@439..440 " "
                  CURLY_GROUP@440..463
                    L_CURLY@440..441 "{"
                    WORD@441..462 "10.1145/359340.359342"
                    R_CURLY@462..463 "}"
                  COMMA@463..464 ","
                WHITESPACE@464..469 "\n    "
                FIELD@469..493
                  NAME@469..476 "journal"
                  WHITESPACE@476..477 " "
                  EQ@477..478 "="
                  WHITESPACE@478..479 " "
                  CURLY_GROUP@479..492
                    L_CURLY@479..480 "{"
                    WORD@480..487 "Commun."
                    WHITESPACE@487..488 " "
                    WORD@488..491 "ACM"
                    R_CURLY@491..492 "}"
                  COMMA@492..493 ","
                WHITESPACE@493..498 "\n    "
                FIELD@498..512
                  NAME@498..503 "month"
                  WHITESPACE@503..504 " "
                  EQ@504..505 "="
                  WHITESPACE@505..506 " "
                  CURLY_GROUP@506..511
                    L_CURLY@506..507 "{"
                    WORD@507..510 "feb"
                    R_CURLY@510..511 "}"
                  COMMA@511..512 ","
                WHITESPACE@512..517 "\n    "
                FIELD@517..535
                  NAME@517..522 "pages"
                  WHITESPACE@522..523 " "
                  EQ@523..524 "="
                  WHITESPACE@524..525 " "
                  CURLY_GROUP@525..534
                    L_CURLY@525..526 "{"
                    WORD@526..533 "120-126"
                    R_CURLY@533..534 "}"
                  COMMA@534..535 ","
                WHITESPACE@535..540 "\n    "
                FIELD@540..555
                  NAME@540..548 "numpages"
                  WHITESPACE@548..549 " "
                  EQ@549..550 "="
                  WHITESPACE@550..551 " "
                  CURLY_GROUP@551..554
                    L_CURLY@551..552 "{"
                    INTEGER@552..553 "7"
                    R_CURLY@553..554 "}"
                  COMMA@554..555 ","
                WHITESPACE@555..556 "\n"
                R_DELIM@556..557 "}"

        "#]],
    );
}

#[test]
fn test_cuesta_2002() {
    check(
        r#"@INPROCEEDINGS{Cuesta02,
  author = {Cuesta, Carlos E. and de la Fuente, Pablo and Barrio-Sol\órzano, Manuel and Beato, Encarnaci\ón},
  title = {{Coordination in a Reflective Architecture Description Language}},
  booktitle = {{Proceedings of the 5th International Conference on Coordination Models and Languages (COORDINATION'02)}},
  year = 2002,
  editor = {Arbab, Fahrad and Talcott, Carolyn},
  pages = {141--148},
  address = {York, United Kingdom},
  organization = {},
  publisher = {Springer},
  volume = {},
  number = {},
  series = {Lecture Notes in Computer Science 2315},
  month = apr,
  note = {}
}"#,
        expect![[r#"
            ROOT@0..617
              ENTRY@0..617
                TYPE@0..14 "@INPROCEEDINGS"
                L_DELIM@14..15 "{"
                NAME@15..23 "Cuesta02"
                COMMA@23..24 ","
                WHITESPACE@24..27 "\n  "
                FIELD@27..136
                  NAME@27..33 "author"
                  WHITESPACE@33..34 " "
                  EQ@34..35 "="
                  WHITESPACE@35..36 " "
                  CURLY_GROUP@36..135
                    L_CURLY@36..37 "{"
                    WORD@37..43 "Cuesta"
                    COMMA@43..44 ","
                    WHITESPACE@44..45 " "
                    WORD@45..51 "Carlos"
                    WHITESPACE@51..52 " "
                    WORD@52..54 "E."
                    WHITESPACE@54..55 " "
                    WORD@55..58 "and"
                    WHITESPACE@58..59 " "
                    WORD@59..61 "de"
                    WHITESPACE@61..62 " "
                    WORD@62..64 "la"
                    WHITESPACE@64..65 " "
                    WORD@65..71 "Fuente"
                    COMMA@71..72 ","
                    WHITESPACE@72..73 " "
                    WORD@73..78 "Pablo"
                    WHITESPACE@78..79 " "
                    WORD@79..82 "and"
                    WHITESPACE@82..83 " "
                    WORD@83..93 "Barrio-Sol"
                    COMMAND@93..101
                      COMMAND_NAME@93..101 "\\órzano"
                    COMMA@101..102 ","
                    WHITESPACE@102..103 " "
                    WORD@103..109 "Manuel"
                    WHITESPACE@109..110 " "
                    WORD@110..113 "and"
                    WHITESPACE@113..114 " "
                    WORD@114..119 "Beato"
                    COMMA@119..120 ","
                    WHITESPACE@120..121 " "
                    WORD@121..130 "Encarnaci"
                    COMMAND@130..134
                      COMMAND_NAME@130..134 "\\ón"
                    R_CURLY@134..135 "}"
                  COMMA@135..136 ","
                WHITESPACE@136..139 "\n  "
                FIELD@139..214
                  NAME@139..144 "title"
                  WHITESPACE@144..145 " "
                  EQ@145..146 "="
                  WHITESPACE@146..147 " "
                  CURLY_GROUP@147..213
                    L_CURLY@147..148 "{"
                    CURLY_GROUP@148..212
                      L_CURLY@148..149 "{"
                      WORD@149..161 "Coordination"
                      WHITESPACE@161..162 " "
                      WORD@162..164 "in"
                      WHITESPACE@164..165 " "
                      WORD@165..166 "a"
                      WHITESPACE@166..167 " "
                      WORD@167..177 "Reflective"
                      WHITESPACE@177..178 " "
                      WORD@178..190 "Architecture"
                      WHITESPACE@190..191 " "
                      WORD@191..202 "Description"
                      WHITESPACE@202..203 " "
                      WORD@203..211 "Language"
                      R_CURLY@211..212 "}"
                    R_CURLY@212..213 "}"
                  COMMA@213..214 ","
                WHITESPACE@214..217 "\n  "
                FIELD@217..336
                  NAME@217..226 "booktitle"
                  WHITESPACE@226..227 " "
                  EQ@227..228 "="
                  WHITESPACE@228..229 " "
                  CURLY_GROUP@229..335
                    L_CURLY@229..230 "{"
                    CURLY_GROUP@230..334
                      L_CURLY@230..231 "{"
                      WORD@231..242 "Proceedings"
                      WHITESPACE@242..243 " "
                      WORD@243..245 "of"
                      WHITESPACE@245..246 " "
                      WORD@246..249 "the"
                      WHITESPACE@249..250 " "
                      WORD@250..253 "5th"
                      WHITESPACE@253..254 " "
                      WORD@254..267 "International"
                      WHITESPACE@267..268 " "
                      WORD@268..278 "Conference"
                      WHITESPACE@278..279 " "
                      WORD@279..281 "on"
                      WHITESPACE@281..282 " "
                      WORD@282..294 "Coordination"
                      WHITESPACE@294..295 " "
                      WORD@295..301 "Models"
                      WHITESPACE@301..302 " "
                      WORD@302..305 "and"
                      WHITESPACE@305..306 " "
                      WORD@306..315 "Languages"
                      WHITESPACE@315..316 " "
                      WORD@316..333 "(COORDINATION'02)"
                      R_CURLY@333..334 "}"
                    R_CURLY@334..335 "}"
                  COMMA@335..336 ","
                WHITESPACE@336..339 "\n  "
                FIELD@339..351
                  NAME@339..343 "year"
                  WHITESPACE@343..344 " "
                  EQ@344..345 "="
                  WHITESPACE@345..346 " "
                  LITERAL@346..350
                    INTEGER@346..350 "2002"
                  COMMA@350..351 ","
                WHITESPACE@351..354 "\n  "
                FIELD@354..400
                  NAME@354..360 "editor"
                  WHITESPACE@360..361 " "
                  EQ@361..362 "="
                  WHITESPACE@362..363 " "
                  CURLY_GROUP@363..399
                    L_CURLY@363..364 "{"
                    WORD@364..369 "Arbab"
                    COMMA@369..370 ","
                    WHITESPACE@370..371 " "
                    WORD@371..377 "Fahrad"
                    WHITESPACE@377..378 " "
                    WORD@378..381 "and"
                    WHITESPACE@381..382 " "
                    WORD@382..389 "Talcott"
                    COMMA@389..390 ","
                    WHITESPACE@390..391 " "
                    WORD@391..398 "Carolyn"
                    R_CURLY@398..399 "}"
                  COMMA@399..400 ","
                WHITESPACE@400..403 "\n  "
                FIELD@403..422
                  NAME@403..408 "pages"
                  WHITESPACE@408..409 " "
                  EQ@409..410 "="
                  WHITESPACE@410..411 " "
                  CURLY_GROUP@411..421
                    L_CURLY@411..412 "{"
                    WORD@412..420 "141--148"
                    R_CURLY@420..421 "}"
                  COMMA@421..422 ","
                WHITESPACE@422..425 "\n  "
                FIELD@425..458
                  NAME@425..432 "address"
                  WHITESPACE@432..433 " "
                  EQ@433..434 "="
                  WHITESPACE@434..435 " "
                  CURLY_GROUP@435..457
                    L_CURLY@435..436 "{"
                    WORD@436..440 "York"
                    COMMA@440..441 ","
                    WHITESPACE@441..442 " "
                    WORD@442..448 "United"
                    WHITESPACE@448..449 " "
                    WORD@449..456 "Kingdom"
                    R_CURLY@456..457 "}"
                  COMMA@457..458 ","
                WHITESPACE@458..461 "\n  "
                FIELD@461..479
                  NAME@461..473 "organization"
                  WHITESPACE@473..474 " "
                  EQ@474..475 "="
                  WHITESPACE@475..476 " "
                  CURLY_GROUP@476..478
                    L_CURLY@476..477 "{"
                    R_CURLY@477..478 "}"
                  COMMA@478..479 ","
                WHITESPACE@479..482 "\n  "
                FIELD@482..505
                  NAME@482..491 "publisher"
                  WHITESPACE@491..492 " "
                  EQ@492..493 "="
                  WHITESPACE@493..494 " "
                  CURLY_GROUP@494..504
                    L_CURLY@494..495 "{"
                    WORD@495..503 "Springer"
                    R_CURLY@503..504 "}"
                  COMMA@504..505 ","
                WHITESPACE@505..508 "\n  "
                FIELD@508..520
                  NAME@508..514 "volume"
                  WHITESPACE@514..515 " "
                  EQ@515..516 "="
                  WHITESPACE@516..517 " "
                  CURLY_GROUP@517..519
                    L_CURLY@517..518 "{"
                    R_CURLY@518..519 "}"
                  COMMA@519..520 ","
                WHITESPACE@520..523 "\n  "
                FIELD@523..535
                  NAME@523..529 "number"
                  WHITESPACE@529..530 " "
                  EQ@530..531 "="
                  WHITESPACE@531..532 " "
                  CURLY_GROUP@532..534
                    L_CURLY@532..533 "{"
                    R_CURLY@533..534 "}"
                  COMMA@534..535 ","
                WHITESPACE@535..538 "\n  "
                FIELD@538..588
                  NAME@538..544 "series"
                  WHITESPACE@544..545 " "
                  EQ@545..546 "="
                  WHITESPACE@546..547 " "
                  CURLY_GROUP@547..587
                    L_CURLY@547..548 "{"
                    WORD@548..555 "Lecture"
                    WHITESPACE@555..556 " "
                    WORD@556..561 "Notes"
                    WHITESPACE@561..562 " "
                    WORD@562..564 "in"
                    WHITESPACE@564..565 " "
                    WORD@565..573 "Computer"
                    WHITESPACE@573..574 " "
                    WORD@574..581 "Science"
                    WHITESPACE@581..582 " "
                    INTEGER@582..586 "2315"
                    R_CURLY@586..587 "}"
                  COMMA@587..588 ","
                WHITESPACE@588..591 "\n  "
                FIELD@591..603
                  NAME@591..596 "month"
                  WHITESPACE@596..597 " "
                  EQ@597..598 "="
                  WHITESPACE@598..599 " "
                  LITERAL@599..602
                    NAME@599..602 "apr"
                  COMMA@602..603 ","
                WHITESPACE@603..606 "\n  "
                FIELD@606..616
                  NAME@606..610 "note"
                  WHITESPACE@610..611 " "
                  EQ@611..612 "="
                  WHITESPACE@612..613 " "
                  CURLY_GROUP@613..615
                    L_CURLY@613..614 "{"
                    R_CURLY@614..615 "}"
                  WHITESPACE@615..616 "\n"
                R_DELIM@616..617 "}"

        "#]],
    )
}
