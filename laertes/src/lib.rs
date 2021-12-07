use proc_macro::{self, TokenStream, TokenTree::*, Delimiter, token_stream};
use quote::{quote, format_ident};
use std::collections::HashSet;

use std::fs::File;
use std::io::{self, BufRead};


struct Act {
    name: String,
    scenes: Vec<String>
}

fn set_from_wordlist(path: &str) -> HashSet<String> {
    let file: File = File::open(path).expect(&format!("couldn't find wordlist: {}", path));
    let mut lines = io::BufReader::new(file).lines();
    let mut set = HashSet::new();

    while let Some(Ok(s)) = lines.next() {
        for word in s.split(' ') {
            set.insert(word.to_string());
        }
    }

    set
}

fn constant_value(stream: &mut token_stream::IntoIter) -> isize {

    let mut total = 1;
    while let Some(Ident(id)) = stream.next() {
    }

    total
}

#[proc_macro]
pub fn shakespeare(input: TokenStream) -> TokenStream {
    let mut out = quote!();
    let mut input = input.into_iter().peekable();

    // word lists
    let character_names = set_from_wordlist("laertes/src/wordlists/character.wordlist");
    dbg!(&character_names);

    loop {
        if let Ident(next) = input.peek().expect("expected character name or first act!") {
            dbg!(next);
            if next.to_string().to_lowercase() == "act" {
                break;
            }
            let mut name: String = String::new();
            // get all words of name until punctuation (names don't have punctuation right?)
            while let Some(Ident(word)) = input.next() {
                name.push_str(&word.to_string());
            }
            let name_ident = format_ident!("{}", name);
            out.extend(quote!(let #name_ident: isize = 0;));
            let stack_ident = format_ident!("{}_stack", name);
            out.extend(quote!(let #stack_ident: Vec<isize> = Vec::new();));
            // get rid of description
            while let Some(Ident(word)) = input.next() {}
        } else {
            dbg!(input.peek());
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

    // keep going until you're out of identifiers
    while let Some(tree) = input.next() {
        if let Ident(id) = tree {
            // get the first word of the statement
            let first = id.to_string().to_lowercase();
            if first == "act" {
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

            if first == "scene" {
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
        }

        else if let Group(grp) = tree {
            assert!(grp.delimiter() == Delimiter::Bracket, "Brackets are the only allowed grouping!");

            let mut direction = grp.stream().into_iter();
            let direction_type = direction.next().expect("can't get stage direction!").to_string();

            if direction_type == "Enter" {
                let mut name = String::new();
                while let Some(tree) = direction.next() {
                    if let Ident(id) = tree {
                        let word = id.to_string();
                        if character_names.contains(&word) {
                            name.push_str(&word);
                            continue;
                        }
                    }

                    stage.insert(name.clone());
                    name.clear();
                }
                stage.insert(name.clone());
                dbg!(&stage);
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
