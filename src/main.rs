use std::env;
use std::fs;
use std::path;

fn main() {
    let args: env::Args = env::args();
    if args.len() > 1 {
        for arg in args.skip(1) {
            classify_files_in(path::Path::new(&arg));
        }
    } else {
        classify_files_in(path::Path::new("."));
    }
}

/// Classify the files by financial year in the given directory.
fn classify_files_in(path: &path::Path) {
    assert!(
        path.try_exists().expect("directory does not exist"),
        "{:?} does not exist",
        path
    );
    assert!(path.is_dir(), "{:?} not a directory", path);

    for entry in path.read_dir().expect("could not read directory") {
        if let Ok(entry) = entry {
            let entry_path = entry.path();
            if entry_path.is_file() {
                match get_fy(&entry_path) {
                    Ok(fy) => place(&entry_path, fy),
                    Err(e) => println!(
                        "Could not get FY for {}. Leaving in place: {}",
                        entry.path().display(),
                        e
                    ),
                }
            }
        }
    }
}

fn place(path: &path::Path, fy: u16) {
    println!("Placing {} in {}", path.display(), fy);

    let base_dir = path.parent().expect("file has no parent");
    let file_name = path.file_name().expect("file does not have a name");
    let dest_dir = base_dir.join(format!("{}FY", fy));

    if !dest_dir.exists() {
        println!("directory {:?} doesn't exit, creating it", &dest_dir);
        fs::create_dir(&dest_dir).expect("could not create directory");
    }

    if !dest_dir.is_dir() {
        println!("{:?} is not a directory?", &dest_dir);
    }
    assert!(dest_dir.is_dir(), "{:?} is not a directory", &dest_dir);

    let dest = dest_dir.join(file_name);
    assert!(!dest.exists(), "{:?} already exists", dest);

    fs::rename(&path, &dest).expect("could not move file");
}

/// Extract the financial year from the file name.
fn get_fy(file_path: &path::Path) -> Result<u16, String> {
    if !file_path.is_file() {
        return Err(String::from("Not a file"));
    }

    let file_name = file_path.file_stem();
    if file_name.is_none() {
        return Err(String::from("No file name"));
    }

    let name_string = file_name
        .unwrap()
        .to_os_string()
        .into_string()
        .expect("could convert to string");
    println!("Processing file name: {:?}", file_path.file_name().unwrap());

    let candidate = name_string.split_terminator('_').last();
    if candidate.is_none() {
        return Err(String::from("Incorrect file name format"));
    }

    let candidate_name = candidate.unwrap();

    match candidate_name.len() {
        6 => get_fy_fy_year_only(&candidate_name),
        7 => process_month_and_year(&candidate_name),
        9 => get_fy_full_date(&candidate_name),
        _ => Err(String::from("File name does not end with date")),
    }
}

/// Get the financial year for dates with just a year and the "FY" suffix. For example "2022FY".
fn get_fy_fy_year_only(date: &str) -> Result<u16, String> {
    if !date[4..6].eq("FY") {
        return Err(String::from(format!("Date is not an FY: {}", date)));
    }
    match date[0..4].parse::<u16>() {
        Ok(year) => return Ok(year),
        Err(e) => Err(format!(
            "Could not parse year {:?}: {}",
            date,
            e.to_string()
        )),
    }
}

/// Get the financial year from a full date (whose format is DDMMMYYYY).
fn get_fy_full_date(date: &str) -> Result<u16, String> {
    let day_str = &date[0..2];
    match date[0..2].parse::<u8>() {
        Ok(_) => process_month_and_year(&date[2..9]),
        Err(e) => Err(format!(
            "Could not parse day of month {:?}: {}",
            day_str,
            e.to_string()
        )),
    }
}

/// Get the financial year from a date with just month and year.
fn process_month_and_year(date: &str) -> Result<u16, String> {
    let offset = get_month_offset(&date[0..3])?;
    let date_str = &date[3..7];
    match date_str.parse::<u16>() {
        Ok(year) => return Ok(year + offset as u16),
        Err(e) => Err(format!(
            "Could not parse year {:?}: {}",
            date_str,
            e.to_string()
        )),
    }
}

/// Gets the offset for each month. The offset (0 for January to June and 1 for July to December)
/// should be added to the current year to get the corresponding financial year. The month is
/// expected to be the first three characters of their name, capitalised.
fn get_month_offset(month: &str) -> Result<i8, String> {
    match month {
        "JAN" => Ok(0),
        "FEB" => Ok(0),
        "MAR" => Ok(0),
        "APR" => Ok(0),
        "MAY" => Ok(0),
        "JUN" => Ok(0),
        "JUL" => Ok(1),
        "AUG" => Ok(1),
        "SEP" => Ok(1),
        "OCT" => Ok(1),
        "NOV" => Ok(1),
        "DEC" => Ok(1),
        _ => Err(format!("Month {:?} not recognised", month)),
    }
}

#[cfg(test)]
mod tests {
    use std::collections;
    use std::env;
    use std::fs;
    use std::path;

    use crate::classify_files_in;

    struct TestData {
        base_path: path::PathBuf,
        expected: collections::HashSet<path::PathBuf>,
    }

    impl TestData {
        fn new(base_path: &path::Path) -> Self {
            TestData {
                base_path: path::PathBuf::from(base_path),
                expected: collections::HashSet::new(),
            }
        }

        fn add_file(&mut self, file_name: &str) {
            let sample_path = (*self.base_path).join(file_name);
            fs::File::options()
                .write(true)
                .create_new(true)
                .open(&sample_path)
                .expect(format!("could not create file {:?}", &sample_path).as_str());
            self.expected.insert((*self.base_path).join(file_name));
        }

        fn add_subdir_file(&mut self, subdir: &str, file_name: &str) {
            let sample_path = (*self.base_path).join(file_name);
            fs::File::options()
                .write(true)
                .create_new(true)
                .open(&sample_path)
                .expect(format!("could not create file {:?}", &sample_path).as_str());
            self.expected
                .insert((*self.base_path).join(subdir).join(file_name));
        }
    }

    #[test]
    fn test_classification() {
        let tempdir = tempfile::tempdir().expect("could not create temp directory");
        let base_path = tempdir.path();
        println!("Temp directory: {:?}", base_path);
        assert!(env::set_current_dir(&base_path).is_ok());

        let mut context: TestData = TestData::new(base_path);
        context.add_subdir_file("2021FY", "text_21JAN2021.txt");
        context.add_subdir_file("2021FY", "text_27FEB2021.txt");
        context.add_subdir_file("2021FY", "text_03MAR2021.txt");
        context.add_subdir_file("2020FY", "text_10APR2020.txt");
        context.add_subdir_file("2020FY", "text_more_10MAY2020.txt");
        context.add_subdir_file("2020FY", "text_JUN2020");
        context.add_subdir_file("2023FY", "text_10JUL2022.txt");
        context.add_subdir_file("2022FY", "text_12AUG2021.txt");
        context.add_subdir_file("2023FY", "14SEP2022.txt");
        context.add_subdir_file("2021FY", "text_20OCT2020.txt");
        context.add_subdir_file("2021FY", "text_08NOV2020");
        context.add_subdir_file("2022FY", "text_01DEC2021.txt");
        context.add_subdir_file("2020FY", "text_2020FY.txt");
        context.add_file("text.txt");
        context.add_file("text_other_2015fy.txt");
        context.add_file("text_abcdFY.txt");
        context.add_file("text_A1JAN2020.txt");
        context.add_file("text_10NAN2020.txt");

        classify_files_in(base_path);

        let mut acc: collections::HashSet<path::PathBuf> = collections::HashSet::new();
        collect_files(&base_path, &mut acc);

        for p in &acc {
            println!("Found file {:?}", p);
        }
        for p in &context.expected {
            println!("Expecting file {:?}", p);
        }

        assert_eq!(&acc, &context.expected);
    }

    fn collect_files(path: &path::Path, acc: &mut collections::HashSet<path::PathBuf>) {
        for entry in path.read_dir().expect("could not read directory") {
            let entry_path = entry.expect("could not read entry").path();
            if entry_path.is_file() {
                acc.insert(entry_path);
            } else if entry_path.is_dir() {
                collect_files(&entry_path, acc);
            }
        }
    }
}
