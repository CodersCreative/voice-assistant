# SADE Voice Assistant

SADE is a python voice assistant which is was made to be easy to set up, customize and learn from.

## Installation
### Download LLM Model

[Install Ollama](https://ollama.ai/download)
[Pull Orca-Mini](https://ollama.ai/library/orca-mini)

On Linux:
```
curl https://ollama.ai/install.sh | sh
ollama pull orca-mini:3b
```

### Run SADE Voice Assistant

```
cd path/to/voice-assistant/
pip install -r requirements.txt
cd assistant
python3 main.py
```

## Contributing

Pull requests are welcome. For major changes, please open an issue first
to discuss what you would like to change.

## Libraries
It was made using these libraries:\
[Langchain](https://www.langchain.com/)\
[Ollama](https://ollama.ai/)\
[Sounddevice](https://pypi.org/project/sounddevice/)\
[Soundfile](https://pypi.org/project/soundfile/)\
[Coqui TTS](https://github.com/coqui-ai/TTS)\
[Faster Whisper](https://github.com/SYSTRAN/faster-whisper)

## License

[MIT](https://choosealicense.com/licenses/mit/)