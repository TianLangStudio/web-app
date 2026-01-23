use std::sync::OnceLock;
use tokio::runtime::Handle;
use tokio_cron_scheduler::{Job, JobBuilder, JobScheduler, JobToRunAsync};
use uuid::Uuid;
use log::info;

pub static EVERY_1_HOUR_CRON: &str = "0 0 * * * *";
pub type AsyncJob = JobToRunAsync;
static JOB_SCHEDULE: OnceLock<JobScheduler> = OnceLock::new();
async fn job_scheduler() -> anyhow::Result<&'static JobScheduler> {
    let schedule = JOB_SCHEDULE.get_or_init(|| {
        tokio::task::block_in_place(move || Handle::current().block_on(JobScheduler::new()))
            .expect("failed to get job schedule")
    });
    Ok(schedule)
}
async fn start() -> anyhow::Result<()> {
    let scheduler = job_scheduler().await?;
    scheduler.start().await?;
    Ok(())
}

pub async fn init() -> anyhow::Result<()> {
    if JOB_SCHEDULE.get().is_none() {
        start().await?;
        info!("TlTask is initiated successfully");
    }
    Ok(())
}
async fn create_schedule_job(cron: &str, job: Box<AsyncJob>) -> anyhow::Result<Job> {
    let job = JobBuilder::new()
        .with_timezone(chrono_tz::Asia::Shanghai)
        .with_cron_job_type()
        .with_schedule(cron)?
        .with_run_async(job)
        .build()?;
    Ok(job)
}
pub async fn add_job_to_scheduler(cron: &str, job: Box<AsyncJob>) -> anyhow::Result<Uuid> {
    init().await?;
    let schedule = job_scheduler().await?;
    let job = create_schedule_job(cron, job).await?;
    let job_id = schedule.add(job).await?;
    Ok(job_id)
}

pub async fn remove_job_from_scheduler(job_id: &Uuid) -> anyhow::Result<()> {
    job_scheduler().await?.remove(job_id).await?;
    Ok(())
}
