use chrono::NaiveDate;
use untis::LessonCode;

// pub enum DiffType {
//     Status,
//     Teacher,
//     Room,
//     Subject,
//     Time,
//     ActivityType,
// }

// pub struct Diffs(pub Vec<DiffType>);

#[derive(Debug, Clone)]
pub enum Diff<'a> {
    Added(&'a untis::Lesson),
    Changed {
        from: &'a db::models::Lesson,
        to: &'a untis::Lesson,
    },
}

impl Diff<'_> {
    pub fn find<'a>(
        prev_schedule: &'a [db::models::Lesson],
        new_schedule: &'a [untis::Lesson],
    ) -> Vec<Diff<'a>> {
        let mut diff = Vec::new();

        for new_lesson in new_schedule {
            if let Some(prev_lesson) = prev_schedule
                .iter()
                .find(|l| l.lesson_id as usize == new_lesson.id)
            {
                if !Self::lessons_equal(prev_lesson, new_lesson) {
                    diff.push(Diff::Changed {
                        from: prev_lesson,
                        to: new_lesson,
                    });
                }
            } else {
                diff.push(Diff::Added(new_lesson));
            }
        }

        diff
    }

    // Compare DB lesson with Untis lesson
    fn lessons_equal(prev: &db::models::Lesson, new: &untis::Lesson) -> bool {
        let new_type_str = match new.lesson_type {
            untis::LessonType::Lesson => "Unterricht",
            untis::LessonType::OfficeHour => "oh",
            untis::LessonType::Standby => "sb",
            untis::LessonType::BreakSupervision => "bs",
            untis::LessonType::Exam => "ex",
        };

        if prev.lesson_id != new.id as i64
            || prev.date != new.date.0
            || prev.start_time != new.start_time.0
            || prev.end_time != new.end_time.0
            || prev.lesson_code != new.code.to_string()
            || prev.lesson_type.as_deref() != Some(new_type_str)
            || prev.subst_text != new.subst_text
        {
            return false;
        }

        // Compare string sets (order-independent)
        fn sets_equal(a: &[String], b: &[String]) -> bool {
            if a.len() != b.len() {
                return false;
            }
            let mut a_sorted = a.to_vec();
            let mut b_sorted = b.to_vec();
            a_sorted.sort();
            b_sorted.sort();
            a_sorted == b_sorted
        }

        let new_subjects: Vec<String> = new.subjects.iter().map(|s| s.name.clone()).collect();
        let new_teachers: Vec<String> = new.teachers.iter().map(|t| t.name.clone()).collect();
        let new_rooms: Vec<String> = new.rooms.iter().map(|r| r.name.clone()).collect();
        let new_classes: Vec<String> = new.classes.iter().map(|c| c.name.clone()).collect();

        sets_equal(&prev.subjects, &new_subjects)
            && sets_equal(&prev.teachers, &new_teachers)
            && sets_equal(&prev.rooms, &new_rooms)
            && sets_equal(&prev.classes, &new_classes)
    }

    pub fn date(&self) -> NaiveDate {
        match self {
            Diff::Added(lesson) => lesson.date.0,
            Diff::Changed { from, .. } => from.date,
        }
    }

    pub fn start_time(&self) -> untis::Time {
        match self {
            Diff::Added(lesson) => lesson.start_time,
            Diff::Changed { from, .. } => untis::Time(from.start_time),
        }
    }

    pub fn end_time(&self) -> untis::Time {
        match self {
            Diff::Added(lesson) => lesson.end_time,
            Diff::Changed { from, .. } => untis::Time(from.end_time),
        }
    }

    pub fn code(&self) -> &LessonCode {
        match self {
            Diff::Added(lesson) => &lesson.code,
            Diff::Changed { to, .. } => &to.code,
        }
    }
}
