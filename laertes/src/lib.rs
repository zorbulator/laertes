use proc_macro::{self, TokenStream, TokenTree, TokenTree::*};
use quote::{quote, format_ident};
use proc_macro::Ident;
use std::collections::HashMap;

struct Act {
    name: String,
    scenes: Vec<String>
}

#[proc_macro]
pub fn shakespeare(input: TokenStream) -> TokenStream {
    let mut out = quote!();
    dbg!(input.clone());
    let mut input = input.into_iter().peekable();

    let chars: Vec<String> = Vec::new();
    dbg!(input.peek());
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
            out.extend(quote!(let #name_ident = 0;));
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
    let mut act_num: i32 = 0;
    let mut scene_num: i32= 0;

    // the bodies of the scenes
    // made of pairs (ident, body)
    let mut scenes = Vec::new();

    // keep going until you're out of identifiers
    // anything other than an identifier will stop the loop since every statement should start with one
    while let Some(Ident(id)) = input.next() {
        // get the first word of the statement
        let first = id.to_string().to_lowercase();
        if first == "act" {
            act_title.clear();
            // ignore the act number
            while let Some(Ident(number)) = input.next() {}
            while let Some(Ident(word)) = input.next() {
                // get the act title but only alphabetic characters and make them lowercase
                act_title.push_str(word.to_string.to_lowercase().replace(|c: char| !c.is_alphabetic(), ""));
            }

            act_num += 1;
            scene_num = 0;
            acts.push(Act { name: act_title, scenes: Vec::new() });
        }

        if first == "scene" {
            let mut scene_title = String::new();
            while let Some(Ident(number)) = input.next() {}
            while let Some(Ident(word)) = input.next() {
                scene_title.push_str(word.to_string.to_lowercase().replace(|c: char| !c.is_alphabetic(), ""));
            }
            // push this scene to the current act
            acts[acts.len() - 1].scenes.push(scene_title);

            scene_num += 1;

            let scene_ident = format_ident!("a{}s{}", act_num, scene_num);
            scenes.push((scene_ident, quote!()));
            
        }
    }

    // the body of the match
    let body = quote!();
    for i in 0..(scenes.len() - 2) {
        // add all but the last scene to the match
        // .0 is the ident, .1 is the body
        // go to the next scene at the end
        body.extend(quote!(
            Scene::#scenes[i].0 => {
                #scenes[i].1
                current_scene = Scene::#scenes[i+1].0;
            }
        ));
    }

    // add in the very last scene with a break
    // since there is no next scene
    body.extend(quote!(
        #scenes[scenes.len() - 1].0 {
            #scenes[scenes.len() - 1].1
            break;
        }
    ));

    out.extend(quote!(
        let current_scene = Scene::a1s1;
        loop {
            match current_scene {
                #body
            }
        }
    ));


    // make an enum with all the scenes for the main match
    out.extend(quote!(
        enum Scene {
            #(scenes.iter().map(|(a, b)| a).collect()),*
        }
    ));

    out.into()
}
