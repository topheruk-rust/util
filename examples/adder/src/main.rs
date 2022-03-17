use std::num::*;

#[derive(Debug)]
struct Student {
    name: String,
    gpa: f32,
}

fn main() -> Result<(), ParseFloatError> {
    let students = vec![
        "Bogdan 3.1",
        "Wallace 2.3",
        "Lidiya 3.5",
        "Kyle 3.9",
        "Anatoliy 4.0",
    ];

    // -- IMPERATIVE

    // let mut good_students = vec![];

    // for s in students {
    //     let mut s = s.splitn(2, ' ');
    //     if let (Some(name), Some(gpa)) = (s.next(), s.next()) {
    //         let name = name.to_string();
    //         let gpa = gpa.parse()?;
    //         if gpa >= 3.5 {
    //             good_students.push(Student { name, gpa });
    //         }
    //     };
    // }

    // for s in good_students {
    //     println!("{:?}", s);
    // }

    // -- COMBINATORS

    let x = Some(0);
    let x = x.ok_or(0); //not that useful
    let x = x.ok();

    let good_students = students
        .iter()
        .filter_map(|s| {
            let mut s = s.splitn(2, ' ');
            let name = s.next()?.to_owned();
            let gpa = s.next()?.parse().ok()?;
            Some(Student { name, gpa })
        })
        .filter(|s| s.gpa >= 3.5)
        .collect::<Vec<Student>>();

    // -- RESULTS
    for s in good_students {
        println!("{:?}", s);
    }

    Ok(())
}
