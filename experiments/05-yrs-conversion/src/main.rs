fn main() -> anyhow::Result<()> {
    let input: serde_json::Value = serde_json::from_reader(std::io::stdin())?;
    let output = yrs_conversion_experiment::request(input)?;
    serde_json::to_writer(std::io::stdout(), &output)?;
    Ok(())
}
