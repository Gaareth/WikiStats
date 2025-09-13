import requests
from celery.signals import task_failure, after_task_publish
import smtplib
from email.mime.text import MIMEText
import os
from celery.utils.log import get_task_logger
from utils import set_task_status

from tasks import logger
import traceback as tb


ALERT_EMAIL_ADDRESS = os.getenv("ALERT_EMAIL_ADDRESS")


SMTP_SERVER = os.getenv("SMTP_SERVER")
SMTP_PORT = int(os.getenv("SMTP_PORT", 587))
SMTP_USER = os.getenv("SMTP_USER")
SMTP_PASSWORD = os.getenv("SMTP_PASSWORD")
SMTP_ENV_VARS = [
    "SMTP_SERVER",
    "SMTP_PORT",
    "SMTP_USER",
    "SMTP_PASSWORD",
    "ALERT_EMAIL_ADDRESS",
]

MAIL_GUN_DOMAIN = os.getenv("MAIL_GUN_DOMAIN")
MAILGUN_API_KEY = os.getenv("MAILGUN_API_KEY")
MAIL_GUN_ENV_VARS = [
    "MAIL_GUN_DOMAIN",
    "MAILGUN_API_KEY",
    "ALERT_EMAIL_ADDRESS",
]


def check_missing_env_vars(vars_to_check):
    return [var for var in vars_to_check if not os.getenv(var)]


if check_missing_env_vars(SMTP_ENV_VARS) and check_missing_env_vars(MAIL_GUN_ENV_VARS):
    print(
        f"No email configuration found. Please set either SMTP or Mailgun environment variables. "
        f"Missing: {', '.join(SMTP_ENV_VARS + MAIL_GUN_ENV_VARS)}"
    )


def send_mail_smtp(subject, body_text):
    missing_env_vars = check_missing_env_vars(SMTP_ENV_VARS)
    if missing_env_vars:
        msg = f"Error: No SMTP email configuration found. Missing: {', '.join(missing_env_vars)}"
        logger.error(msg)
        print(msg)
        return

    msg = MIMEText(body_text)
    msg["Subject"] = subject
    msg["From"] = SMTP_USER
    msg["To"] = ALERT_EMAIL_ADDRESS

    try:
        with smtplib.SMTP(SMTP_SERVER, SMTP_PORT) as server:
            server.starttls()
            server.login(SMTP_USER, SMTP_PASSWORD)
            result = server.sendmail(SMTP_USER, [ALERT_EMAIL_ADDRESS], msg.as_string())
            print(result)
    except Exception as e:
        logger.error(f"Failed to send failure alert email: {e}")
        print(f"Failed to send failure alert email: {e}")


def send_mail_mailgun(subject, body_text):
    missing_env_vars = check_missing_env_vars(MAIL_GUN_ENV_VARS)
    if missing_env_vars:
        msg = f"Error: No mailgun email configuration found. Missing: {', '.join(missing_env_vars)}"
        logger.error(msg)
        print(msg)
        return

    response = requests.post(
        f"https://api.mailgun.net/v3/{MAIL_GUN_DOMAIN}/messages",
        auth=("api", MAILGUN_API_KEY),
        data={
            "from": f"Test <postmaster@{MAIL_GUN_DOMAIN}>",
            "to": ALERT_EMAIL_ADDRESS,
            "subject": subject,
            "text": body_text,
        },
    )
    if response.status_code != 200:
        msg = f"Failed to send failure alert email via Mailgun: {response.text}"
        logger.error(msg)
        print(msg)


@task_failure.connect
def task_failed_handler(sender=None, headers=None, body=None, **kwargs):
    """Send an email alert when a task fails."""


    if check_missing_env_vars(SMTP_ENV_VARS):
        if check_missing_env_vars(MAIL_GUN_ENV_VARS):
            msg = "Error: No email configuration found. Please set either SMTP or Mailgun environment variables."
            logger.error(msg)
            print(msg)
            return
        else:
            send_mail = send_mail_mailgun
            logger.info("Using Mailgun for sending failure alert emails.")
    else:
        send_mail = send_mail_smtp
        logger.info("Using SMTP for sending failure alert emails.")


    task_name = sender.name if sender else "Unknown"
    exception = kwargs.get("exception", "No exception info")
    traceback = kwargs.get("traceback", "")
    if traceback and isinstance(traceback, str):
        formatted_traceback = traceback
    elif traceback:
        formatted_traceback = "".join(tb.format_tb(traceback))
    else:
        formatted_traceback = "No traceback available"

    traceback = formatted_traceback

    subject = f"Celery Task Failed: {task_name}"
    body_text = f"Task: {task_name}\nException: {exception}\nTraceback:\n{traceback}"
    logger.info(f"Sending failure alert to {ALERT_EMAIL_ADDRESS} for task {task_name}")
    send_mail(subject, body_text)


# @after_task_publish.connect
# def task_sent_handler(sender=None, headers=None, body=None, **kwargs):
#     print(f"Task {sender} queued with id {headers.get('id')} {body}")
#     if sender == "tasks.process_wiki":
#         wiki = body[0] 
#         dump_date = body[1] 
#         set_task_status({
#                 "name": wiki,
#                 "dumpDate": dump_date,
#                 "status": "QUEUED",
#                 "startedAt": None,
#                 "finishedAt": None,
#                 "message": None,
#             })


if __name__ == "__main__":
    # For testing purposes
    send_mail_mailgun("Test Email", "This is a test email from the task failure handler.")
