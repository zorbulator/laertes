use std::cmp::Ordering;
use proc_macro::{self, TokenStream, TokenTree::*, Delimiter, token_stream};
use quote::{quote, format_ident};
use std::collections::HashSet;

use std::iter::Peekable;


struct Act {
    name: String,
    scenes: Vec<String>
}

fn constant_value(stream: &mut Peekable<token_stream::IntoIter>) -> isize {
    let nouns: HashSet<&str> = HashSet::from_iter(include_str!("wordlists/nouns.wordlist").lines());
    let negative_nouns: HashSet<&str> = HashSet::from_iter(include_str!("wordlists/negative_nouns.wordlist").lines());
    let possessive: HashSet<&str> = HashSet::from(["thine", "thy", "your", "my", "mine", "his", "her"]);
    let zero = HashSet::from(["zero", "nothing"]);
    
    let mut total = 1;
    while let Some(Ident(id)) = stream.next() {
        // ignore possessives like "your sorry little codpiece"
        if possessive.contains(id.to_string().as_str()) {
            continue;
        }

        // also ignore a/an
        if id.to_string() == "a" || id.to_string() == "an" {
            continue;
        }

        if nouns.contains(id.to_string().as_str()) {
            if negative_nouns.contains(id.to_string().as_str()) {
                total *= -1;
            }
            return total;
        }

        if zero.contains(id.to_string().as_str()) {
            return 0;
        }

        total *= 2;
    }

    total
}

// takes the stream and the characters and evaluates a whole expression
fn evaluate_expression(stream: &mut Peekable<token_stream::IntoIter>, speaking: &str, other: &str) -> proc_macro2::TokenStream {
    // "the" indicates some kind of function like "
    if let Some(Ident(first_ident)) = stream.peek() {
        let first_word = first_ident.to_string();

        // ignore "a" in case someone writes something like "you are as stupid as a half-witted coward!"
        if first_word == "a" {
            stream.next();
            return evaluate_expression(stream, speaking, other);
        }

        if first_word == "twice" {
            stream.next();
            let arg = evaluate_expression(stream, speaking, other);
            return quote!(((#arg) * 2)).into();
        }

        let second_person_reflexive = HashSet::from(["thyself", "yourself", "you", "thou"]);

        if second_person_reflexive.contains(first_word.as_str()) {
            stream.next(); // get rid of the word
            let ident = format_ident!("{}", &other);
            return proc_macro2::TokenTree::Ident(ident).into();
        }

        let first_person_reflexive = HashSet::from(["me", "myself", "I"]);

        if first_person_reflexive.contains(first_word.as_str()) {
            stream.next();
            let ident = format_ident!("{}", &speaking);
            return proc_macro2::TokenTree::Ident(ident).into();
        }

        let character_names: HashSet<&str> = HashSet::from_iter(include_str!("wordlists/character.wordlist").lines());


        if character_names.contains(first_word.as_str()) {
            let mut name = String::from(&first_word);
            stream.next();
            while character_names.contains(stream.peek().expect("unexpected end").to_string().as_str()) {
                name.push_str(&stream.next().unwrap().to_string());
            }
            let ident = format_ident!("{}", &name);
            return proc_macro2::TokenTree::Ident(ident).into();
        }

        // "the" indicates this is some kind of function like "the difference between thyself and Hamlet"
        if first_word == "the" {
            stream.next();
            let operation = stream.next().expect("missing operation").to_string();
            let second_word = stream.next().expect("missing second word of operation").to_string();


            match operation.as_str() {
                "cube" => {
                    assert_eq!(second_word, "of", "the word after 'cube' must be 'of'");
                    let arg = evaluate_expression(stream, speaking, other);
                    return quote!((#arg).pow(3)).into();
                },
                "difference" => {
                    assert_eq!(second_word, "between", "the word after 'difference' must be 'between'");
                    let arg1 = evaluate_expression(stream, speaking, other);
                    assert_eq!(stream.next().expect("expected 'and' to separate arguments but got nothing").to_string(), "and", "arguments must be separated by 'and'");
                    let arg2 = evaluate_expression(stream, speaking, other);
                    return quote!(((#arg1) - (#arg2))).into();
                },
                "factorial" => {
                    assert_eq!(second_word, "of", "the word after 'factorial' must be 'of'");
                    let arg = evaluate_expression(stream, speaking, other);
                    return quote!(((1..=#arg).product())).into();
                },
                "product" => {
                    assert_eq!(second_word, "of", "the word after 'product' must be 'of'");
                    let arg1 = evaluate_expression(stream, speaking, other);
                    assert_eq!(stream.next().expect("expected 'and' to separate arguments but got nothing").to_string(), "and", "arguments must be separated by 'and'");
                    let arg2 = evaluate_expression(stream, speaking, other);
                    return quote!(((#arg1) * (#arg2))).into();
                },
                "quotient" => {
                    assert_eq!(second_word, "between", "the word after 'quotient' must be 'between'");
                    let arg1 = evaluate_expression(stream, speaking, other);
                    assert_eq!(stream.next().expect("expected 'and' to separate arguments but got nothing").to_string(), "and", "arguments must be separated by 'and'");
                    let arg2 = evaluate_expression(stream, speaking, other);
                    return quote!(((#arg1) / (#arg2))).into();
                },
                // original is "remainder of the quotient between"
                // this is a breaking change but that just sounds bad and is more than 2 words
                "modulus" => {
                    assert_eq!(second_word, "of", "the word after 'modulus' must be 'of'");
                    let arg1 = evaluate_expression(stream, speaking, other);
                    assert_eq!(stream.next().expect("expected 'and' to separate arguments but got nothing").to_string(), "and", "arguments must be separated by 'and'");
                    let arg2 = evaluate_expression(stream, speaking, other);
                    return quote!(((#arg1) % (#arg2))).into();
                },
                "square" => {
                    assert_eq!(second_word, "of", "the word after 'square' must be 'of'");
                    let arg = evaluate_expression(stream, speaking, other);
                    return quote!((#arg).pow(2)).into();
                },
                // another change to a single word
                "root" => {
                    assert_eq!(second_word, "of", "the word after 'square' must be 'of'");
                    let arg = evaluate_expression(stream, speaking, other);
                    return quote!(((#arg as f64).sqrt() as isize)).into();
                },
                "sum" => {
                    assert_eq!(second_word, "of", "the word after 'sum' must be 'of'");
                    let arg1 = evaluate_expression(stream, speaking, other);
                    assert_eq!(stream.next().expect("expected 'and' to separate arguments but got nothing").to_string(), "and", "arguments must be separated by 'and'");
                    let arg2 = evaluate_expression(stream, speaking, other);
                    return quote!(((#arg1) + (#arg2))).into();
                },
                "xor" => {
                    assert_eq!(second_word, "of", "the word after 'xor' must be 'of'");
                    let arg1 = evaluate_expression(stream, speaking, other);
                    assert_eq!(stream.next().expect("expected 'and' to separate arguments but got nothing").to_string(), "and", "arguments must be separated by 'and'");
                    let arg2 = evaluate_expression(stream, speaking, other);
                    return quote!(((#arg1) ^ (#arg2))).into();
                },
                _ => { panic!("invalid operation: {}", operation); }
            }
        }
    }

    // if it's not anything else, it's a regular constant
    let value = constant_value(stream);
    quote!(#value).into()
}


#[proc_macro]
pub fn shakespeare(input: TokenStream) -> TokenStream {
    let mut out = quote!();
    let mut input = input.into_iter().peekable();

    // word lists
    let character_names: HashSet<&str> = HashSet::from_iter(include_str!("wordlists/character.wordlist").lines());
    let second_person: HashSet<&str> = HashSet::from(["thee", "thou", "you"]);
    let be: HashSet<&str> = HashSet::from(["am", "are", "art", "be", "is"]);
    let positive_comparisons: HashSet<&str> = HashSet::from_iter(include_str!("wordlists/positive_comparative.wordlist").lines());
    let negative_comparisons: HashSet<&str> = HashSet::from_iter(include_str!("wordlists/negative_comparative.wordlist").lines());

    loop {
        if let Ident(next) = input.peek().expect("expected character name or first act!") {
            if next.to_string().to_lowercase() == "act" {
                break;
            }
            let mut name: String = String::new();
            // get all words of name until punctuation (names don't have punctuation right?)
            while let Some(Ident(word)) = input.next() {
                name.push_str(&word.to_string());
            }
            let name_ident = format_ident!("{}", name);
            out.extend(quote!(let mut #name_ident: isize = 0;));
            let stack_ident = format_ident!("{}_stack", name);
            out.extend(quote!(let mut #stack_ident: Vec<isize> = Vec::new();));
            // get rid of description
            while let Some(Ident(_)) = input.next() {}
        } else {
            panic!("unexpected non-identifier instead of character name or first act!");
        }
    }

    let mut acts: Vec<Act> = Vec::new();
    let mut act_title = String::new();


    // start at act 0 because act keyword will increment it to 1
    let mut act_num: u32 = 0;
    let mut scene_num: u32= 0;

    // the bodies of the scenes
    // made of pairs (ident, body)
    let mut scenes = Vec::new();

    // the characters on stage
    let mut stage: HashSet<String> = HashSet::new();

    // the character currently talking; hopefully this always has a value
    let mut current_character = String::new();

    // keep going until you're out of identifiers
    while let Some(tree) = input.next() {
        if let Ident(id) = tree {
            // get the first word of the statement
            let first = id.to_string();
            if first == "Act" {
                act_title.clear();
                // ignore the act number
                while let Some(Ident(_)) = input.next() {}
                while let Some(Ident(word)) = input.next() {
                    // get the act title but only alphabetic characters and make them lowercase
                    act_title.push_str(&word.to_string().to_lowercase().replace(|c: char| !c.is_alphabetic(), ""));
                }

                act_num += 1;
                scene_num = 0;
                acts.push(Act { name: act_title.clone(), scenes: Vec::new() });
            }

            if first == "Scene" {
                let mut scene_title = String::new();
                while let Some(Ident(_)) = input.next() {}
                while let Some(Ident(word)) = input.next() {
                    scene_title.push_str(&word.to_string().to_lowercase().replace(|c: char| !c.is_alphabetic(), ""));
                }
                // push this scene to the current act
                let end = acts.len() - 1;
                acts[end].scenes.push(scene_title);

                scene_num += 1;

                let scene_ident = format_ident!("a{}s{}", act_num, scene_num);
                scenes.push((scene_ident, quote!()));
            }
            // character name; this means a new character is talking
            if character_names.contains(first.as_str()) {
                let mut name = first.clone();
                while let Some(Ident(word)) = input.next() {
                    name.push_str(&word.to_string());
                }
                assert!(stage.contains(&name), "character not on stage: {}", name);
                current_character = name;
            }

            // start with you/thou means this is an assignment
            if second_person.contains(first.to_lowercase().as_str()) {

                let peek = &input.peek().expect("unexpected end").to_string();

                // "You lying stupid fatherless big smelly half-witted coward!" and
                // "Thou art the sum of a fine Lord and a summer day"
                // are both valid sentences, so just ignore the "are"/"art"
                if be.contains(peek.as_str()) {
                    input.next(); // get rid of the "are"
                }

                let peek = &input.peek().expect("unexpected end").to_string();

                if peek == "as" {
                    assert_eq!(input.next().expect("unexpected end").to_string(), "as".to_string(), "assignment must have 'as' twice like 'you are as _ as _");
                    input.next(); // get rid of the adjective
                    assert_eq!(input.next().expect("unexpected end").to_string(), "as".to_string(), "assignment must have 'as' twice like 'you are as _ as _");
                }

                let peek = &input.peek().expect("unexpected end").to_string();

                // also ignore a/an
                if peek == "a" || peek == "an" {
                    input.next();
                }

                // get all the characters other than the current one
                let just_me = HashSet::from([current_character.clone()]);
                let mut other_characters = stage.difference(&just_me);
                assert_eq!(other_characters.clone().collect::<HashSet<&String>>().len(), 1, "two characters must be on the stage to use {}", &first);
                let other_character = other_characters.next().expect("no other character on stage for assignment!");
                let new_value = evaluate_expression(&mut input, &current_character, &other_character);
                let other_ident = format_ident!("{}", other_character);
                let latest_scene = scenes.len() - 1;
                scenes[latest_scene].1.extend(quote!(
                    #other_ident = #new_value;
                ));
            }

            // "Open your mind" == print numerical value, but we'll allow anything starting with "open"
            // "Speak your mind" == print unicode value
            if first == "Open" || first == "Speak" || first == "Listen" {
                // "open your mind" reads a character so check if the phrase contains "mind"
                let mut contains_mind = false;
                // ignore the rest of the identifiers
                while let Some(Ident(id)) = input.next() {
                    if id.to_string() == "mind" {
                        contains_mind = true;
                    }
                }

                // get all the characters other than the current one
                let just_me = HashSet::from([current_character.clone()]);
                let mut other_characters = stage.difference(&just_me);
                assert_eq!(other_characters.clone().collect::<HashSet<&String>>().len(), 1, "two characters must be on the stage to use {}", &first);
                let other_character = other_characters.next().expect("no other character on stage for assignment!");
                let other_ident = format_ident!("{}", other_character);
                let latest_scene = scenes.len() - 1;

                if first == "Open" {
                    if contains_mind {
                        scenes[latest_scene].1.extend(quote!(
                            let mut line = String::new();
                            std::io::stdin().read_line(&mut line).expect("failed to read character");
                            let c = line.chars().next().expect("got line with no characters!") as isize;
                            #other_ident = c;
                        ));
                    } else {
                        scenes[latest_scene].1.extend(quote!(
                            println!("{}", #other_ident);
                        ));
                    }
                } else if first == "Speak" {
                    scenes[latest_scene].1.extend(quote!(
                        print!("{}", std::char::from_u32(#other_ident.abs() as u32).expect(&format!("Invalid character: {} has value {}", "#other_ident", #other_ident)));
                    ));
                } else if first == "Listen" {
                    scenes[latest_scene].1.extend(quote!(
                        use std::io::BufRead;
                        let num: isize = std::io::stdin()
                            .lock()
                            .lines()
                            .next()
                            .expect("stdin should be available")
                            .expect("couldn't read from stdin")
                            .trim()
                            .parse()
                            .expect("input was not an integer");
                        #other_ident = num;
                    ));
                }
            }

            // "if so" / "if not" for conditionals
            if first == "If" {
                let latest_scene = scenes.len() - 1;
                let next = input.next().expect("unexpected stop").to_string();

                if next == "so" {
                    scenes[latest_scene].1.extend(quote!(
                        final_cond = cond;
                    ));
                } else if next == "not" {
                    scenes[latest_scene].1.extend(quote!(
                        final_cond = !cond;
                    ));
                } else {
                    panic!("'If' must be followed by 'so' or 'not'");
                }

                assert!(matches!(input.next().expect("unexpected stop"), Punct(_)), "If {} must be followed by punctuation", &next);
            }

            // comparison like "am I better than you?"
            if be.contains(first.to_lowercase().as_str()) {
                let just_me = HashSet::from([current_character.clone()]);
                let mut other_characters = stage.difference(&just_me);
                assert_eq!(other_characters.clone().collect::<HashSet<&String>>().len(), 1, "two characters must be on the stage to use {}", &first);
                let other_character = other_characters.next().expect("no other character on stage for assignment!");
                let first_value = evaluate_expression(&mut input, &current_character, &other_character);
                // the type of comparison to check for
                let test: Ordering;
                let comparison_word = input.next().expect("unexpected stop").to_string();

                if comparison_word == "as" {
                    // like "are you as _ as _?"
                    input.next();
                    assert_eq!(input.next().expect("unexpected stop").to_string(), "as");
                    test = Ordering::Equal;
                } else if negative_comparisons.contains(comparison_word.as_str()) {
                    // get rid of word like "than"
                    input.next();
                    test = Ordering::Less;
                } else if positive_comparisons.contains(comparison_word.as_str()) {
                    input.next();
                    test = Ordering::Greater;
                } else {
                    panic!("invalid comparison");
                }

                let second_value = evaluate_expression(&mut input, &current_character, &other_character);

                let latest_scene = scenes.len() - 1;

                let test_ident = match test {
                    Ordering::Equal   => quote!(std::cmp::Ordering::Equal),
                    Ordering::Less    => quote!(std::cmp::Ordering::Less),
                    Ordering::Greater => quote!(std::cmp::Ordering::Greater)
                };

                scenes[latest_scene].1.extend(quote!(
                    cond = (#first_value).cmp(&(#second_value)) == #test_ident;
                ));

                // there's still punctuation at the end
                //assert!(matches!(input.next().expect("unexpected stop"), Punct(_)), "Condition must be followed by punctuation");

            }

            // "let us" / "we shall" are gotos
            if first.to_lowercase() == "let" || first.to_lowercase() == "we" {
                // get rid of three words like "us proceed to" or "shall return to"
                // I could check this, but maybe people want to be creative and change the words
                for _ in 0..3 { input.next(); }

                let latest_scene = scenes.len() - 1;

                let peek = input.peek().expect("unexpected stop").to_string().to_lowercase();

                // goto another scene/act by number
                if peek == "scene" || peek == "act" {
                    input.next();
                    let roman_numeral = input.next().expect("unexpected stop").to_string();
                    let jump_number: u32 = decode_roman_numeral(&roman_numeral);
                    let scene_ident;

                    // if it's a scene then jump to the same act but the same number
                    if peek == "scene" {
                        scene_ident = format_ident!("a{}s{}", act_num, jump_number);
                    } else {
                        // if it's an act then go to scene 1
                        scene_ident = format_ident!("a{}s{}", jump_number, 1u32);
                    }

                    scenes[latest_scene].1.extend(quote!(
                        if (final_cond) {
                            current_scene = Scene::#scene_ident;
                            continue;
                        }
                        final_cond = true;
                    ));

                    input.next(); // get rid of punctuation
                } else {
                    // otherwise it's the actual title of a scene
                    
                    let mut jump_to = String::new();
                    while let Some(Ident(word)) = input.next() {
                        // get the act title but only alphabetic characters and make them lowercase
                        jump_to.push_str(&word.to_string().to_lowercase().replace(|c: char| !c.is_alphabetic(), ""));
                    }

                    // act/scene to jump to
                    let (mut a, mut s) = (0, 0);
                    'outer: for (act_index, act) in acts.iter().enumerate() {
                        if act.name == jump_to {
                            a = act_index + 1;
                            s = 1;
                            break 'outer;
                        }
                        for (scene_index, scene_title) in act.scenes.iter().enumerate() {
                            if scene_title == &jump_to {
                                a = act_index + 1;
                                s = scene_index + 1;
                                break 'outer;
                            }
                        }
                    }

                    assert_ne!(a, 0, "act or scene name to jump to didn't match");
                    let scene_ident = format_ident!("a{}s{}", a, s);

                    scenes[latest_scene].1.extend(quote!(
                        if final_cond {
                            current_scene = Scene::#scene_ident;
                            continue;
                        }
                        final_cond = true;
                    ));
                }
            }

            if first == "Remember" || first == "Recall" {
                let just_me = HashSet::from([current_character.clone()]);
                let mut other_characters = stage.difference(&just_me);
                assert_eq!(other_characters.clone().collect::<HashSet<&String>>().len(), 1, "two characters must be on the stage to use {}", &first);
                let other_character = other_characters.next().expect("no other character on stage for assignment!");
                let other_ident = format_ident!("{}", other_character);
                let other_stack_ident = format_ident!("{}_stack", other_character);
                let latest_scene = scenes.len() - 1;

                if first == "Recall" {
                    scenes[latest_scene].1.extend(quote!(
                        #other_ident = #other_stack_ident.pop().expect("Tried to pop empty stack!");
                    ));

                    // the rest doesn't matter
                    while let Some(Ident(_)) = input.next() {}
                } else {
                    let first_value = evaluate_expression(&mut input, &current_character, &other_character);
                    scenes[latest_scene].1.extend(quote!(
                        #other_stack_ident.push((#first_value));
                    ));
                }
            }
        }

        // brackets used for stuff like enter/exit
        else if let Group(grp) = tree {
            assert!(grp.delimiter() == Delimiter::Bracket, "Brackets are the only allowed grouping!");

            let mut direction = grp.stream().into_iter().peekable();
            let direction_type = direction.next().expect("can't get stage direction!").to_string();

            if direction_type == "Enter" || direction_type == "Exit" || direction_type == "Exeunt" {
                // are there character names?
                if let None = direction.peek() {
                    match direction_type.as_str() {
                        "Exeunt" => {
                            // exeunt with no characters; exit all and continue to next loop
                            stage.clear();
                            continue;
                        }
                        _ => {
                            panic!("Enter or exit with no characters! Use Exeunt to exit all");
                        }
                    }
                }
                
                let mut name = String::new();
                while let Some(tree) = direction.next() {
                    if let Ident(id) = tree {
                        let word = id.to_string();
                        if character_names.contains(word.as_str()) {
                            name.push_str(&word);
                            continue;
                        }
                    }

                    // only runs if this isn't a name
                    match direction_type.as_str() {
                        "Enter" => {
                            stage.insert(name.clone());
                        },
                        "Exeunt" => {
                            stage.remove(&name);
                        },
                        "Exit" => {
                            panic!("can only exit one character; did you mean Exeunt?");
                        },
                        _ => { unreachable!() }
                    }
                    name.clear();
                }
                // what to do with the last name
                match direction_type.as_str() {
                    "Enter" => {
                        stage.insert(name.clone());
                    },
                    // if it's not enter it's exit or exeunt
                    _ => {
                        stage.remove(&name);
                    },
                }
            }
        }
    }

    // the body of the match
    let mut body = quote!();
    for i in 0..(scenes.len() - 1) {
        // add all but the last scene to the match
        // .0 is the ident, .1 is the body
        // go to the next scene at the end
        let this_scene_ident = &scenes[i].0;
        let this_scene_body = &scenes[i].1;
        let next_scene_ident = &scenes[i+1].0;
        body.extend(quote!(
            Scene::#this_scene_ident => {
                #this_scene_body
                current_scene = Scene::#next_scene_ident;
            }
        ));
    }

    // add in the very last scene with a break
    // since there is no next scene
    let end = scenes.len() - 1;
    let final_scene_ident = &scenes[end].0;
    let final_scene_body = &scenes[end].1;
    body.extend(quote!(
        Scene::#final_scene_ident => {
            #final_scene_body
            break;
        }
    ));

    out.extend(quote!(
        let mut current_scene = Scene::a1s1;
        // the condition from statements like "am I better than you?"
        let mut cond = true;
        // the actual condition set by "if so,"
        let mut final_cond = true;
        loop {
            match current_scene {
                #body
            }
        }
    ));


    // make an enum with all the scenes for the main match
    let scene_names = scenes.iter().map(|(a, _)| a);
    out.extend(quote!(
        enum Scene {
            #(#scene_names),*
        }
    ));

    out.into()
}

struct RomanNumeral {
    symbol: &'static str,
    value: u32
}

 const NUMERALS: [RomanNumeral; 13] = [
    RomanNumeral {symbol: "M",  value: 1000},
    RomanNumeral {symbol: "CM", value: 900},
    RomanNumeral {symbol: "D",  value: 500},
    RomanNumeral {symbol: "CD", value: 400},
    RomanNumeral {symbol: "C",  value: 100},
    RomanNumeral {symbol: "XC", value: 90},
    RomanNumeral {symbol: "L",  value: 50},
    RomanNumeral {symbol: "XL", value: 40},
    RomanNumeral {symbol: "X",  value: 10},
    RomanNumeral {symbol: "IX", value: 9},
    RomanNumeral {symbol: "V",  value: 5},
    RomanNumeral {symbol: "IV", value: 4},
    RomanNumeral {symbol: "I",  value: 1}
];

// roman numeral function I just copied from rosettacode.org because I don't want to do it myself
fn decode_roman_numeral(roman: &str) -> u32 {
    match NUMERALS.iter().find(|num| roman.starts_with(num.symbol)) {
        Some(num) => num.value + decode_roman_numeral(&roman[num.symbol.len()..]),
        None => 0, // if string empty, add nothing
    }
}
