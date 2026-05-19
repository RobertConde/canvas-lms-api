use canvas_lms_api::Canvas;

#[tokio::main]
async fn main() -> canvas_lms_api::Result<()> {
    let base_url = std::env::var("CANVAS_URL")
        .expect("CANVAS_URL must be set (e.g. https://canvas.example.edu)");
    let token = std::env::var("CANVAS_TOKEN").expect("CANVAS_TOKEN must be set");

    let canvas = Canvas::new(&base_url, &token)?;

    let courses = canvas.get_courses().collect_all().await?;
    println!("Found {} course(s)\n", courses.len());

    for course in courses {
        let name = course.name.as_deref().unwrap_or("(unnamed)");
        println!("Course {}: {}", course.id, name);

        match course.get_assignments().collect_all().await {
            Ok(assignments) if assignments.is_empty() => println!("  (no assignments)"),
            Ok(assignments) => {
                for a in assignments {
                    let title = a.name.as_deref().unwrap_or("(unnamed)");
                    let points = a
                        .points_possible
                        .map(|p| format!("{p} pts"))
                        .unwrap_or_else(|| "ungraded".into());
                    println!("  - {title} ({points})");
                }
            }
            Err(e) => println!("  (could not fetch assignments: {e})"),
        }
        println!();
    }

    Ok(())
}
