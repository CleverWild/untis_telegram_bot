use chrono::TimeZone;

use crate::{datetime::Date, error::Error, jsonrpc, params, resources::*, Session};

/// Client for accessing the Untis API. Can be constructed by [`Client::login()`](Self::login) or [`School::client_login()`](School::client_login).
///
/// # Example
/// ```rust
/// let result = untis::Client::login("server.webuntis.com", "school", "username", "password");
/// match result {
///     Err(err) => println!("{}", err),
///     Ok(client) => {
///         let info = client.session();
///     }
/// }
/// ```
pub struct Client {
    rpc_client: jsonrpc::Client,
    pub session: Session,
}

impl Client {
    /// Асинхронный метод для создания новой сессии.
    pub async fn login(
        server: &str,
        school: &str,
        username: &str,
        password: &str,
    ) -> Result<Self, Error> {
        let params = params::AuthenticateParams {
            client: "untis-rs",
            user: username,
            password,
        };
        let mut rpc_client = jsonrpc::Client::new(&make_untis_url(server, school));
        let session: Session = rpc_client.request("authenticate", params).await?;
        Ok(Self {
            rpc_client,
            session,
        })
    }

    /// Returns the active session.
    pub fn session(&self) -> &Session {
        &self.session
    }

    /// Возвращает последний раз, когда любое расписание в этой школе было обновлено.
    pub async fn last_update_time(&mut self) -> Result<chrono::DateTime<chrono::Utc>, Error> {
        let ts: i64 = self.rpc_client.request("getLatestImportTime", ()).await?;
        Ok(chrono::Utc.timestamp_millis_opt(ts).unwrap())
    }

    /// Возвращает статусные данные для отображения расписания.
    pub async fn status_data(&mut self) -> Result<StatusData, Error> {
        self.rpc_client.request("getStatusData", ()).await
    }

    /// Получает текущий учебный год.
    pub async fn current_schoolyear(&mut self) -> Result<Schoolyear, Error> {
        self.rpc_client.request("getCurrentSchoolyear", ()).await
    }

    /// Получает список всех учебных годов.
    pub async fn schoolyears(&mut self) -> Result<Vec<Schoolyear>, Error> {
        self.rpc_client.request("getSchoolyears", ()).await
    }

    /// Получает каникулы в текущем учебном году.
    pub async fn holidays(&mut self) -> Result<Vec<Holiday>, Error> {
        self.rpc_client.request("getHolidays", ()).await
    }

    /// Получает список комнат в школе пользователя.
    pub async fn rooms(&mut self) -> Result<Vec<Room>, Error> {
        self.rpc_client.request("getRooms", ()).await
    }

    /// Retrieves the list of classes in the user's school.
    pub async fn classes(&mut self) -> Result<Vec<Class>, Error> {
        self.rpc_client.request("getKlassen", ()).await
    }

    /// Retrieves the list of subjects in the user's school.
    pub async fn subjects(&mut self) -> Result<Vec<Subject>, Error> {
        self.rpc_client.request("getSubjects", ()).await
    }

    /// Retrieves the list of teachers in the user's school.
    pub async fn teachers(&mut self) -> Result<Vec<Teacher>, Error> {
        self.rpc_client.request("getTeachers", ()).await
    }

    /// Retrieves the list of students in the user's school.
    pub async fn students(&mut self) -> Result<Vec<Student>, Error> {
        self.rpc_client.request("getStudents", ()).await
    }

    /// Retrieves the user's own timetable between now and a given date.
    pub async fn own_timetable_until(&mut self, end_date: &Date) -> Result<Vec<Lesson>, Error> {
        self.own_timetable_between(&Date::today(), end_date).await
    }

    /// Retrieves the users's own timetable for the current week.
    pub async fn own_timetable_current_week(&mut self) -> Result<Vec<Lesson>, Error> {
        self.own_timetable_for_week(&Date::today()).await
    }

    /// Retrieves the users's own timetable for the week that a given date is in.
    pub async fn own_timetable_for_week(&mut self, date: &Date) -> Result<Vec<Lesson>, Error> {
        self.own_timetable_between(&date.relative_week_begin(), &date.relative_week_end()).await
    }

    /// Retrieves the users's own timetable between two dates.
    pub async fn own_timetable_between(
        &mut self,
        start_date: &Date,
        end_date: &Date,
    ) -> Result<Vec<Lesson>, Error> {
        self.timetable_between(
            &self.session.person_id.clone(),
            &self.session.person_type.clone(),
            start_date,
            end_date,
        ).await
    }

    /// Retrieves an element's timetable between now and a given date.
    pub async fn timetable_until(
        &mut self,
        id: &usize,
        ty: &ElementType,
        end_date: &Date,
    ) -> Result<Vec<Lesson>, Error> {
        self.timetable_between(id, ty, &Date::today(), end_date).await
    }

    /// Retrieves an element's timetable for the current week.
    pub async fn timetable_current_week(
        &mut self,
        id: &usize,
        ty: &ElementType,
    ) -> Result<Vec<Lesson>, Error> {
        self.timetable_for_week(id, ty, &Date::today()).await
    }

    /// Retrieves an element's timetable for the week that a given date is in.
    pub async fn timetable_for_week(
        &mut self,
        id: &usize,
        ty: &ElementType,
        date: &Date,
    ) -> Result<Vec<Lesson>, Error> {
        self.timetable_between(
            id,
            ty,
            &date.relative_week_begin(),
            &date.relative_week_end(),
        ).await
    }

    /// Retrieves an element's own timetable between two dates.
    pub async fn timetable_between(
        &mut self,
        id: &usize,
        ty: &ElementType,
        start_date: &Date,
        end_date: &Date,
    ) -> Result<Vec<Lesson>, Error> {
        let params = params::TimetableParams {
            options: &params::TimetableParamsOpts {
                element: &params::TimetableParamsElem { id, ty },
                start_date,
                end_date,
                show_booking: &true,
                show_info: &true,
                show_subst_text: &true,
                show_ls_text: &true,
                show_ls_number: &true,
                show_student_group: &true,
                class_fields: &["id", "name"],
                room_fields: &["id", "name"],
                subject_fields: &["id", "name"],
                teacher_fields: &["id", "name"],
            },
        };
        self.rpc_client.request("getTimetable", params).await
    }

    /// Retrieves the list of departments in the user's school.
    pub async fn departments(&mut self) -> Result<Vec<Department>, Error> {
        self.rpc_client.request("getDepartments", ()).await
    }

    fn logout(&mut self) -> Result<(), Error> {
        // self.rpc_client.request("logout", ())
        Ok(())
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        _ = self.logout();
    }
}

impl School {
    pub async fn client_login(&self, username: &str, password: &str) -> Result<Client, Error> {
        Client::login(&self.server, &self.login_name, username, password).await
    }
}

fn make_untis_url(server: &str, school: &str) -> String {
    format!("https://{server}/WebUntis/jsonrpc.do?school={school}")
}
