use std::fs;
use tokio::task::JoinSet;

#[cfg(test)]
mod test;

pub struct FileWithLines {
    text: String,
    line_breaks: Vec<usize>,
}

impl FileWithLines {
    pub fn get_ith_line(&self, i: usize) -> &str {
        if self.line_breaks.len() < i + 1 {
            panic!("too much");
        } else if self.line_breaks.len() == i + 1 {
            return &self.text[self.line_breaks[i]..];
        }

        let res = &self.text[self.line_breaks[i]..self.line_breaks[i + 1]];
        if res.ends_with('\n') {
            &self.text[self.line_breaks[i]..self.line_breaks[i + 1] - 1]
        } else {
            res
        }
    }

    pub fn len(&self) -> usize {
        self.line_breaks.len() - 2
    }

    pub async fn from_files(files: Vec<String>) -> Vec<FileWithLines> {
        let mut futures: JoinSet<FileWithLines> = JoinSet::new();
        for file in files {
            futures.spawn(load_file(file.clone()));
        }

        let mut result_vec: Vec<FileWithLines> = Vec::new();

        while let Some(result) = futures.join_next().await {
            let text = result.unwrap();
            result_vec.push(text);
        }

        result_vec
    }
}

fn read_file_to_string(path: &String) -> String {
    fs::read_to_string(path)
        .expect(format!("Should have been able to read the file={}", path).as_str())
}

fn get_line_breaks(text_str: &String) -> Vec<usize> {
    let mut line_breaks: Vec<usize> = Vec::new();
    line_breaks.push(0);
    let mut find_text = &text_str[0..];
    while let Some(next) = find_text.find('\n') {
        match line_breaks.last() {
            None => line_breaks.push(next + 1),
            Some(l) => line_breaks.push(l + next + 1),
        };
        find_text = &find_text[next + 1..];
    }
    line_breaks.push(text_str.len() - 1);
    line_breaks
}

async fn load_file(path: String) -> FileWithLines {
    let text = read_file_to_string(&path);
    let line_breaks: Vec<usize> = get_line_breaks(&text);
    FileWithLines { text, line_breaks }
}