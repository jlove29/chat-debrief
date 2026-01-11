function downloadChat() {
    if (location.href.includes('gemini.google.com')) {
        downloadGemini();
    } else if (location.href.includes('google.com/search')) {
        downloadAim();
    } else {
        console.log("%c UNSUPPORTED SITE", "color: white; background: red; padding: 3px; font-weight: bold;");
    }
}

downloadChat();