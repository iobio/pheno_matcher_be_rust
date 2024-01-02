use std::collections::HashMap;
use csv::Reader;

pub fn create_population(csv_url: String) -> HashMap<String, HashMap<String, String>> {
    //This will take a csv file and create a hashmap where the numId is the key and the value is a hashmap of the other attributes
    let mut population: HashMap<String, HashMap<String, String>> = HashMap::new();
    //Read the csv file
    let mut reader = Reader::from_path(csv_url).unwrap();
    //Iterate through the rows
    for result in reader.records() {
        let record = result.unwrap();
        //Create a hashmap for each row
        let mut individual: HashMap<String, String> = HashMap::new();
        //Iterate through the columns
        individual.insert("ID".to_string(), record[0].to_string());
        individual.insert("Dx/Udx".to_string(), record[1].to_string());
        //if Genes is NONE, None, or none, then set it to an empty string
        let mut genes = record[2].to_string();
        //remove any periods or empty spaces from the genes
        genes = genes.replace(".", "");
        if genes == "NONE" || genes == "None" || genes == "none" {
            genes = "".to_string();
        }
        individual.insert("Genes".to_string(), genes);
        individual.insert("Clin diagnosis".to_string(), record[3].to_string());
        let mut terms = record[4].to_string();
        //remove any periods or empty spaces from the terms
        terms = terms.replace(".", "");
        //if terms is NONE, None, or none, then set it to an empty string
        if terms == "NONE" || terms == "None" || terms == "none" {
            terms = "".to_string();
        }
        individual.insert("Terms".to_string(), terms);
        individual.insert("HPO_Names".to_string(), record[5].to_string());
        
        //Add the individual to the population hashmap
        population.insert(individual.get("ID").unwrap().to_string(), individual);
    }
    population
}