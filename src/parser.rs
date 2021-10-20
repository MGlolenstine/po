use crate::error::Error;
use crate::{AutoComments, Po};

enum CurrentState {
    /// Untranslated
    Msgid,
    /// Translated
    Msgstr(usize),
}

pub fn string_to_langs(path: &str) -> Result<Vec<Po>, Box<dyn std::error::Error>> {
    if std::path::Path::new(path).exists() {
        //? Reading lines for each entry
        let contents = std::fs::read_to_string(path)?;
        let contents = contents.lines();
        let mut overall: Vec<Vec<String>> = vec![];
        let mut entry: Vec<String> = vec![];
        let mut done_with_comments = false;
        for l in contents {
            if l.is_empty() {
                continue;
            }
            if l.starts_with('#') && done_with_comments {
                //? We do this to avoid cloning the vector.
                //? We create a new one, move all elements into it and move the vector.
                let mut e: Vec<String> = vec![];
                e.append(&mut entry);
                overall.push(e);
                entry.clear();
                entry.push(l.to_owned());
                done_with_comments = false;
            } else if !l.starts_with('#') {
                entry.push(l.to_owned());
                done_with_comments = true;
            } else {
                entry.push(l.to_owned());
            }
        }
        overall.push(entry);

        //? Parse into Po
        let mut pos = vec![];
        let mut current_state: Option<CurrentState> = None;
        for entry in overall {
            let mut po = Po::default();
            for line in entry {
                if line.starts_with("#. ") {
                    po.auto_comments.push(AutoComments::ExtractedComments(
                        line.strip_prefix("#. ").unwrap().to_owned(),
                    ));
                    current_state = None;
                } else if line.starts_with("#: ") {
                    if po.reference.is_empty(){
                        po.reference = line.strip_prefix("#: ").unwrap().to_owned();
                    }else{
                        po.reference.push(' ');
                        po.reference.push_str(&line.strip_prefix("#: ").unwrap().to_owned());
                    }
                    current_state = None;
                } else if line.starts_with("#, ") {
                    po.auto_comments.push(AutoComments::Flag(
                        line.strip_prefix("#, ").unwrap().to_owned(),
                    ));
                    current_state = None;
                } else if line.starts_with("#| ") {
                    po.auto_comments.push(AutoComments::Other(
                        line.strip_prefix("#| ").unwrap().to_owned(),
                    ));
                    current_state = None;
                } else if line.starts_with("msgctxt ") {
                    po.msgctxt = line.strip_prefix("msgctxt ").unwrap().to_owned().replace("\"", "");
                    current_state = None;
                } else if line.starts_with("msgid ") {
                    po.msgid = line
                        .strip_prefix("msgid ")
                        .unwrap()
                        .to_owned()
                        .replace("\"", "");
                    current_state = Some(CurrentState::Msgid);
                } else if line.starts_with("msgstr") {
                    if line.strip_prefix("msgstr").unwrap().starts_with('[') {
                        let text = line.strip_prefix("msgstr[").unwrap();
                        let index_pos = text
                            .find(']')
                            .expect("No closing bracket in numbered `msgstr`.");
                        let index = text[..index_pos]
                            .parse::<usize>()
                            .unwrap_or_else(|_|panic!("{} is not a valid usize!", &text[..index_pos]));
                        //? Here it has to be index+2, as we're trying to account for a whitespace after `msgstr[0] `
                        po.msgstr
                            .insert(index, text[index_pos + 2..].to_owned().replace("\"", ""));
                        current_state = Some(CurrentState::Msgstr(index));
                    } else {
                        po.msgstr.insert(
                            0,
                            line.strip_prefix("msgstr ")
                                .unwrap()
                                .to_owned()
                                .replace("\"", ""),
                        );
                        current_state = Some(CurrentState::Msgstr(0));
                    }
                } else if let Some(ref state) = current_state {
                    match state {
                        CurrentState::Msgid => {
                            po.msgid.push('\n');
                            po.msgid.push_str(&line.replace("\"", ""));
                        }
                        CurrentState::Msgstr(index) => {
                            let msg = po.msgstr.get_mut(index).unwrap();
                            msg.push('\n');
                            msg.push_str(&line.replace("\"", ""));
                        }
                    }
                }
            }
            //? If the msgid is empty, we can't use it.
            if !po.msgid.is_empty() {
                pos.push(po);
            }
        }
        Ok(pos)
    } else {
        Err(Box::new(Error::PathDoesNotExist {
            path: path.to_owned(),
        }))
    }
}

pub fn langs_to_string(langs: &[Po]) -> String {
    let mut ret_string = String::new();
    let mut entry_string = String::new();
    for po in langs {
        //? Add translator comments
        if !po.translator_comments.is_empty() {
            entry_string.push_str(&format!("# {}\n", po.translator_comments));
        }
        //? Add automatic comments
        for c in po.auto_comments.iter() {
            match c{
                AutoComments::ExtractedComments(val) => {
                    entry_string.push_str(&format!("#. {}\n", val));
                }
                AutoComments::Flag(val) => {
                    entry_string.push_str(&format!("#, {}\n", val));
                }
                AutoComments::Other(val) => {
                    entry_string.push_str(&format!("#| {}\n", val));
                }
            }
        }
        //? Add references
        if !po.reference.is_empty() {
            entry_string.push_str(&format!("#: {}\n", po.reference));
        }
        //? Add msgctxt
        if !po.msgctxt.is_empty() {
            if po.msgctxt.contains('\n'){
                let str = po.msgctxt.clone();
                let mut splot = str.split('\n');
                let first = splot.next().unwrap();
                entry_string.push_str(&format!("msgctxt \"{}\"\n", first));
                for s in splot{
                    entry_string.push_str(&format!("\"{}\"\n", s));
                }
            }else{
                entry_string.push_str(&format!("msgctxt \"{}\"\n", po.msgctxt));
            }
        }

        //? Add msgid
        if !po.msgid.is_empty() {
            if po.msgid.contains('\n'){
                let str = po.msgid.clone();
                let mut splot = str.split('\n');
                let first = splot.next().unwrap();
                entry_string.push_str(&format!("msgid \"{}\"\n", first));
                for s in splot{
                    entry_string.push_str(&format!("\"{}\"\n", s));
                }
            }else{
                entry_string.push_str(&format!("msgid \"{}\"\n", po.msgid));
            }
        }

        //? Add msgstr
        if !po.msgstr.is_empty() {
            if po.msgstr.len() == 1 {
                let val = po.msgstr.values().next().unwrap().to_owned();
                if val.contains('\n'){
                    let str = val;
                    let mut splot = str.split('\n');
                    let first = splot.next().unwrap();
                    entry_string.push_str(&format!("msgstr \"{}\"\n", first));
                    for s in splot{
                        entry_string.push_str(&format!("\"{}\"\n", s));
                    }
                }else{
                    entry_string.push_str(&format!("msgstr \"{}\"\n", val));
                }
            }else{
                for (k, v) in po.msgstr.iter() {
                    if v.contains('\n'){
                        let str = v.to_owned();
                        let mut splot = str.split('\n');
                        let first = splot.next().unwrap();
                        entry_string.push_str(&format!("msgstr[{}] \"{}\"\n", k, first));
                        for s in splot{
                            entry_string.push_str(&format!("\"{}\"\n", s));
                        }
                    }else{
                        entry_string.push_str(&format!("msgstr[{}] \"{}\"\n", k, v));
                    }
                }
            }
        }
        ret_string.push_str(&entry_string);
        ret_string.push('\n');
        entry_string.clear();
    }
    ret_string
}
