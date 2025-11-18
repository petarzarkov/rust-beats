import os
from google_auth_oauthlib.flow import InstalledAppFlow

# Load .env file if it exists (for local development)
try:
    from dotenv import load_dotenv
    load_dotenv()
except ImportError:
    # python-dotenv not installed, skip loading .env file
    pass

CLIENT_ID = os.environ.get("YOUTUBE_CLIENT_ID")
CLIENT_SECRET = os.environ.get("YOUTUBE_CLIENT_SECRET")
if not all([CLIENT_ID, CLIENT_SECRET]):
    raise ValueError("Missing YOUTUBE_ credentials in environment variables.")

SCOPES = ["https://www.googleapis.com/auth/youtube.upload"]

def get_new_token():
    flow = InstalledAppFlow.from_client_config(
        {
            "web": {
                "client_id": CLIENT_ID,
                "client_secret": CLIENT_SECRET,
                "auth_uri": "https://accounts.google.com/o/oauth2/auth",
                "token_uri": "https://oauth2.googleapis.com/token",
            }
        },
        SCOPES
    )
    
    creds = flow.run_local_server(port=0)
    
    print("\n--- SUCCESS! ---")
    print("Here is your new REFRESH TOKEN. Save this safely!")
    print(creds.refresh_token)

if __name__ == "__main__":
    get_new_token()