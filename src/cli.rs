use clap::Parser;

/// Blazing fast Gemini CLI client for your terminal
///
/// This tool allows you to interact with Google's Gemini language model directly from your terminal.
/// You can ask questions, get responses, and manage your API key with ease. Perfect for developers, researchers, and anyone curious about AI language models.
/// # Examples
///
/// ```bash
/// # Request a response from Gemini
/// gemini-cli --prompt "What is the capital of France?"
///
/// # Set your API key
/// gemini-cli --set-key "YOUR_API_KEY"
///
/// # Use a different model
/// gemini-cli --prompt "Tell me a joke" --model "gemini-2.5-flash-lite"
/// ```
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// The prompt to send to the Gemini model. If not provided, the program will ask for input interactively.
    /// Example: --prompt "What is the capital of France?"
    /// If you want to use the interactive mode, simply run `gemini-cli` without the `--prompt` flag and it will prompt you to enter your question.
    /// This allows you to have a more conversational experience with the Gemini model, making it easier to ask follow-up questions or clarify your queries without needing to re-run the command.
    #[arg(short, long)]
    pub prompt: Option<String>,
    /// The Gemini model to use for generating responses. By default, it uses "gemini-2.5-flash-lite", which is a fast and efficient model suitable for most tasks.
    /// You can specify a different model if you have access to it and want to try out
    /// other capabilities. For example, you might want to use a more powerful model for complex queries or a smaller one for quick responses.
    /// Example: --model "gemini-2.5-flash-lite"
    /// Make sure to check Google's documentation for the available models and their capabilities to choose the one that best fits your needs.
    /// Note: Using more powerful models may result in longer response times and higher costs, so choose wisely based on your requirements.
    /// If you want to see the list of available models, you can refer to Google's API documentation or use their model listing endpoint to get the most up-to-date information on the models you have access to.
    /// Keep in mind that the default model "gemini-2.5-flash-lite" is a great starting point for most users, offering a good balance of speed and performance for a wide range of tasks.
    /// If you have specific needs or want to experiment with different models, feel free to specify the model you want to use with the `--model` flag.
    /// Remember that the choice of model can impact the quality and speed of the responses you receive, so it's worth trying out different options to see which one works best for your use case.
    /// In summary, the `--model` flag allows you to customize your experience with the Gemini API by selecting the model that best suits your needs, whether it's for quick responses or more complex interactions.
    /// For most users, the default "gemini-2.5-flash-lite" model will provide excellent performance for a wide range of tasks, but feel free to explore other models if you have specific requirements or want to experiment with different capabilities.
    /// Keep in mind that the availability of models may depend on your access level and the specific features you need, so be sure to check Google's documentation for the latest information on available models and their capabilities.
    #[arg(short, long, default_value_t = String::from("gemini-2.5-flash-lite"))]
    pub model: String,
    /// Set your API key for accessing the Gemini API. This is required to authenticate your requests and get responses from the model.
    /// You can set your API key using this flag, and it will be saved securely for future use. Once set, you won't need to provide the key again for subsequent requests, making it
    /// convenient to interact with the Gemini API without having to worry about managing your API key manually.
    /// Example: --set-key "YOUR_API_KEY"
    /// If you don't have an API key yet, you can obtain one from the Google Cloud Console by creating a project and enabling the Gemini API. Once you have your API key, you can
    /// use this flag to set it up for the gemini-cli tool, allowing you to start making requests to the Gemini API right away.
    /// Remember to keep your API key secure and avoid sharing it publicly, as it grants access to your Google Cloud resources and can be used to make requests on your behalf. If you suspect that
    /// your API key has been compromised, be sure to revoke it and generate a new one from the Google Cloud Console to protect your account and resources.
    /// In summary, the `--set-key` flag allows you to easily manage your API key for the Gemini API, ensuring that you can authenticate your requests and interact with the model without having to worry about manual key management. Just set your key once, and you're good to go for all your future interactions with the Gemini API through this CLI tool.
    /// Note: If you want to change your API key in the future, simply run the command again with the `--set-key` flag and provide the new key. The tool will update your stored API key accordingly, allowing you to seamlessly switch between different keys if needed.
    #[arg(long)]
    pub set_key: Option<String>,
}
