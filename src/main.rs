#![allow(unused_imports)]
mod custom_jaccard_ic;
mod population;
mod calc_scores;
mod calc_simpheny_score;
use warp::{Filter, path, reply, Rejection, Reply, http::StatusCode, http::Response, hyper::Body, cors};
use std::sync::Arc;
use std::collections::HashMap;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize, de::IntoDeserializer};
use serde_json::Result as SerdeResult;
use hpo::Ontology;

#[tokio::main]
async fn main() {
    // Overarching variables
    let ontology = Arc::new(Ontology::from_binary("/bin_hpo_file").unwrap()); //Production URL
    // let ontology = Arc::new(Ontology::from_binary("/Users/emerson/Documents/Code/pheno_matcher_be_rust/bin_hpo_file").unwrap()); //Development URL

    // URLS PRODUCTION
    const UDN_CSV_URL: &str = "/data/UdnPatients.csv"; //Production URL
    const ORPHA_TSV_URL: &str = "/data/ORPHANETessentials.tsv"; //Production URL
    const DECIPHER_DATA_URL: &str = "/data/DecipherData.csv"; //Production URL
    const GENE_LIST_URL: &str = "/data/gene_list.csv"; //Production URL
    const TERMS_LIST_URL: &str = "/data/term_list.csv"; //Production URL

    // URLS DEVELOPMENT
    // const UDN_CSV_URL: &str = "/Users/emerson/Documents/Code/pheno_matcher_be_rust/data/UdnPatients.csv"; //Development URL
    // const ORPHA_TSV_URL: &str = "/Users/emerson/Documents/Code/pheno_matcher_be_rust/data/ORPHANETessentials.tsv"; //Development URL
    // const DECIPHER_DATA_URL: &str = "/Users/emerson/Documents/Code/pheno_matcher_be_rust/data/DecipherData.csv"; //Development URL
    // const GENE_LIST_URL: &str = "/Users/emerson/Documents/Code/pheno_matcher_be_rust/data/gene_list.csv"; //Development URL
    // const TERMS_LIST_URL: &str = "/Users/emerson/Documents/Code/pheno_matcher_be_rust/data/term_list.csv"; //Development URL

    let udn_population = Arc::new(population::create_udn_population(UDN_CSV_URL.to_string()));
    let orpha_population = Arc::new(population::create_orpha_population(ORPHA_TSV_URL.to_string()));
    let deciper_population = Arc::new(population::create_deciper_population(DECIPHER_DATA_URL.to_string()));

    // The "/" path will return a generic greeting showing that the backend is running okay
    let home = path::end().map(|| {
        let response = Response::builder()
            .status(StatusCode::OK)
            .header("Access-Control-Allow-Origin", "*")
            .body("matcher_be_rusty and running!");
        response.unwrap()
    });

    // The "/check_db" path will return a message indicating whether the database is found or not
    let check_db = path!("check_db").map(|| {
        let db_path = get_db_path();
        let db_conn = Connection::open(db_path);

        let response = match db_conn {
            Ok(_) => Response::builder()
                .status(StatusCode::OK)
                .header("Access-Control-Allow-Origin", "*")
                .body(format!("db found!")),
            Err(error) => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header("Access-Control-Allow-Origin", "*")
                .body(format!("db not found error: {}", error)),
        };
        response.unwrap()
    });

    //Get all the genes for a term by the term id
    let get_genes_for_term = warp::path!("id" / "get_genes" / String)
        .map(|param: String| {
            let db_path = get_db_path();
            let genes = get_genes_for_term(db_path, param);
            let genes = genes.unwrap();
            let genes = serde_json::to_string(&genes).unwrap();

            let response = Response::builder()
                .status(StatusCode::OK)
                .header("Access-Control-Allow-Origin", "*")
                .header("Content-Type", "application/json")
                .body(genes);
            response
    });
    
    //Get all the terms for a gene by the gene id
    let get_terms_for_gene = warp::path!("gene" / "get_terms" / String)
        .map(|param: String| {
            let db_path = get_db_path();
            let terms = get_terms_for_gene(db_path, param);
            let terms = terms.unwrap();
            let terms = serde_json::to_string(&terms).unwrap();

            let response = Response::builder()
                .status(StatusCode::OK)
                .header("Access-Control-Allow-Origin", "*")
                .header("Content-Type", "application/json")
                .body(terms);
            response
    });

    let get_terms_for_null_gene = warp::path!("gene" / "get_terms")
        .map(|| {
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("Access-Control-Allow-Origin", "*")
                .header("Content-Type", "application/json")
                .body("[]");
            response
    });

    //Get a term from a term hpo id
    let get_term_by_id = warp::path("id").and(warp::path::param()).map(|param: String| {
        let db_path = get_db_path();
        let term = get_term_id(db_path, param);
        let term = term.unwrap();
        let term = serde_json::to_string(&term).unwrap();

        let response = Response::builder()
            .status(StatusCode::OK)
            .header("Access-Control-Allow-Origin", "*")
            .header("Content-Type", "application/json")
            .body(term);
        response
    });

    //Get a term from a term name
    let get_term_by_name = warp::path("name").and(warp::path::param()).map(|param: String| {
        let db_path = get_db_path();
        let param = param.replace("%20", " "); //replace %20 with a space, should be the only issue with names
        let term = get_term_name(db_path, param);
        let term = term.unwrap();
        let term = serde_json::to_string(&term).unwrap();

        let response = Response::builder()
            .status(StatusCode::OK)
            .header("Access-Control-Allow-Origin", "*")
            .header("Content-Type", "application/json")
            .body(term);
        response
    });

    //Get a gene from a gene id
    let get_gene_by_id = warp::path!("gene" / "id" / String)
        .map(|param: String| {
            let db_path = get_db_path();
            let gene = get_gene_by_id(db_path, param);
            let gene = gene.unwrap();
            let gene = serde_json::to_string(&gene).unwrap();

            let response = Response::builder()
                .status(StatusCode::OK)
                .header("Access-Control-Allow-Origin", "*")
                .header("Content-Type", "application/json")
                .body(gene);
            response
    });

    //Get gene from gene name
    let get_gene_by_name = warp::path!("gene" / "name" / String)
        .map(|param: String| {
            let db_path = get_db_path();
            let gene = get_gene_by_name(db_path, param);
            let gene = gene.unwrap();
            let gene = serde_json::to_string(&gene).unwrap();

            let response = Response::builder()
                .status(StatusCode::OK)
                .header("Access-Control-Allow-Origin", "*")
                .header("Content-Type", "application/json")
                .body(gene);
            response
    });

    //Get genes from a list of gene names
    let get_genes_from_names = warp::path!("gene" / "names" / String)
        .map(|param: String| {
            //take out any %20 chars if there are any replace with nothing
            let param = param.replace("%20", "");
            //change the string separated by commas into a vector of strings
            let param = param.split(",").map(|s| s.to_string()).collect::<Vec<String>>();
            let db_path = get_db_path();

            let genes = get_genes_from_names(db_path, param);
            let genes = genes.unwrap();
            let genes = serde_json::to_string(&genes).unwrap();

            let response = Response::builder()
                .status(StatusCode::OK)
                .header("Access-Control-Allow-Origin", "*")
                .header("Content-Type", "application/json")
                .body(genes);
            response
    });

    // The "/all/terms/ids" path will return a json of all the terms in the database with the hpo_id as the key
    let get_all_terms_ids = warp::path!("all" / "terms" / "ids").map(|| {
        let db_path = get_db_path();
        let terms = get_all_terms_ids(db_path);
    
        let response = match terms {
            Ok(terms) => {
                let json_terms = serde_json::to_string(&terms).unwrap();
    
                warp::http::Response::builder()
                    .status(StatusCode::OK)
                    .header("Access-Control-Allow-Origin", "*")
                    .header("Content-Type", "application/json")
                    .body(json_terms)  // Convert String directly to Body
                    .unwrap_or_else(|_| warp::http::Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body("Internal server error".into())
                        .unwrap())
            },
            Err(e) => {
                // Log the error or handle it appropriately
                eprintln!("Database error: {:?}", e);
                warp::http::Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header("Access-Control-Allow-Origin", "*")
                    .body("error: db cannot be found".into())
                    .unwrap()
            }
        };
        response
    });

    // The "/all/terms/names" path will return a json of all the terms in the database by name
    let get_all_terms_names = warp::path!("all" / "terms" / "names").map(|| {
        let db_path = get_db_path();
        let terms = get_all_terms_names(db_path);
    
        let response = match terms {
            Ok(terms) => {
                let json_terms = serde_json::to_string(&terms).unwrap();
    
                warp::http::Response::builder()
                    .status(StatusCode::OK)
                    .header("Access-Control-Allow-Origin", "*")
                    .header("Content-Type", "application/json")
                    .body(json_terms)  // Convert String directly to Body
                    .unwrap_or_else(|_| warp::http::Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body("Internal server error".into())
                        .unwrap())
            },
            Err(e) => {
                // Log the error or handle it appropriately
                eprintln!("Database error: {:?}", e);
                warp::http::Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header("Access-Control-Allow-Origin", "*")
                    .body("error: db cannot be found".into())
                    .unwrap()
            }
        };
        response
    });

    //Use the population function to get the population structure from the csv
    let get_orpha_population = warp::path!("orpha_population").map(|| {
        let orpha_population = population::create_orpha_population(ORPHA_TSV_URL.to_string());
        let json_orpha_population = serde_json::to_string(&orpha_population).unwrap();

        let response = warp::http::Response::builder()
            .status(StatusCode::OK)
            .header("Access-Control-Allow-Origin", "*")
            .header("Content-Type", "application/json")
            .body(json_orpha_population)  // Convert String directly to Body
            .unwrap_or_else(|_| warp::http::Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Internal server error".into())
                .unwrap());
        response
    });

    //Use the population function to get the population structure from the csv
    let get_udn_population = warp::path!("udn_population").map(|| {
        let udn_population = population::create_udn_population(UDN_CSV_URL.to_string());
        let json_udn_population = serde_json::to_string(&udn_population).unwrap();

        let response = warp::http::Response::builder()
            .status(StatusCode::OK)
            .header("Access-Control-Allow-Origin", "*")
            .header("Content-Type", "application/json")
            .body(json_udn_population)  // Convert String directly to Body
            .unwrap_or_else(|_| warp::http::Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Internal server error".into())
                .unwrap());
        response
    });

    let get_decipher_population = warp::path!("decipher_population").map(|| {
        let decipher_population = population::create_deciper_population(DECIPHER_DATA_URL.to_string());
        let json_decipher_population = serde_json::to_string(&decipher_population).unwrap();

        let response = warp::http::Response::builder()
            .status(StatusCode::OK)
            .header("Access-Control-Allow-Origin", "*")
            .header("Content-Type", "application/json")
            .body(json_decipher_population)  // Convert String directly to Body
            .unwrap_or_else(|_| warp::http::Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Internal server error".into())
                .unwrap());
        response
    });

    // Get a map of all of the similarity scores for a given set of terms
    let udn_compare = warp::path("compare_udn" )
        .and(warp::path::param())
        .map({
            let ontology = Arc::clone(&ontology);
            let population = Arc::clone(&udn_population);

            move |param: String| {
                let param = param.replace("%20", "");
                let param_string = param.split(",").map(|s| s.to_string()).collect::<Vec<String>>();
                let param_u32 = param_string.iter().map(|s| s.replace("HP:", "").parse::<u32>().unwrap()).collect::<Vec<u32>>();

                let return_map = calc_scores::calc_scores(&ontology, param_u32, &population);
                let return_map = serde_json::to_string(&return_map).unwrap();

                let response = Response::builder()
                    .status(StatusCode::OK)
                    .header("Access-Control-Allow-Origin", "*")
                    .header("Content-Type", "application/json")
                    .body(warp::hyper::Body::from(return_map))
                    .unwrap_or_else(|_| warp::http::Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body("Internal server error".into())
                        .unwrap());
                response
            }
    });

    // Get a map of all of the similarity scores for a given set of terms
    let orpha_compare = warp::path("compare_orpha")
        .and(warp::path::param())
        .map({
            let ontology = Arc::clone(&ontology);
            let population = Arc::clone(&orpha_population);

            move |param: String| {
                let param = param.replace("%20", "");
                let param_string = param.split(",").map(|s| s.to_string()).collect::<Vec<String>>();
                let param_u32 = param_string.iter().map(|s| s.replace("HP:", "").parse::<u32>().unwrap()).collect::<Vec<u32>>();

                let return_map = calc_scores::calc_scores(&ontology, param_u32, &population);
                let return_map = serde_json::to_string(&return_map).unwrap();

                let response = Response::builder()
                    .status(StatusCode::OK)
                    .header("Access-Control-Allow-Origin", "*")
                    .header("Content-Type", "application/json")
                    .body(warp::hyper::Body::from(return_map))
                    .unwrap_or_else(|_| warp::http::Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body("Internal server error".into())
                        .unwrap());
                response
            }
    });

    // Get a map of all of the similarity scores for a given set of terms
    let decipher_compare = warp::path("compare_decipher")
        .and(warp::path::param())
        .map({
            let ontology = Arc::clone(&ontology);
            let population = Arc::clone(&deciper_population);

            move |param: String| {
                let param = param.replace("%20", "");
                let param_string = param.split(",").map(|s| s.to_string()).collect::<Vec<String>>();
                let param_u32 = param_string.iter().map(|s| s.replace("HP:", "").parse::<u32>().unwrap()).collect::<Vec<u32>>();

                let return_map = calc_scores::calc_scores(&ontology, param_u32, &population);
                let return_map = serde_json::to_string(&return_map).unwrap();

                let response = Response::builder()
                    .status(StatusCode::OK)
                    .header("Access-Control-Allow-Origin", "*")
                    .header("Content-Type", "application/json")
                    .body(warp::hyper::Body::from(return_map))
                    .unwrap_or_else(|_| warp::http::Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body("Internal server error".into())
                        .unwrap());
                response
            }
    });

    #[derive(Deserialize)]
    struct SimphenyScoreRequest {
        hit_terms: Vec<String>, // Expecting "HP:12345" format
        gene_symbol: String,
        sim_score: f32,
        num_query_genes: u32,
        num_hpo_terms: u32,
        data_bg: String, // Background data for the score calculation
    }

    let simpheny_score = warp::path("simpheny_score")
        .and(warp::post())
        .and(warp::body::json())
        .map(move |body: SimphenyScoreRequest| {
            let ontology = Arc::clone(&ontology);

            // Default values for the simulation
            let iterations: u32 = 10000; // Number of iterations for the simulation
            let terms_url: &str = TERMS_LIST_URL;
            let genes_url: &str = GENE_LIST_URL;
            // Terms need the prefix "HP:" removed and then parsed to u32
            let terms_cleaned: Vec<u32> = body.hit_terms.iter()
                .map(|term| term.replace("HP:", "").parse::<u32>().unwrap_or_else(|_| {
                    eprintln!("Error parsing term ID: {}", term);
                    0 // Default value in case of error
                }))
                .collect();
            
            // We aren't borrowing anything but the ontology because we don't need the other variables after this point
            let simpheny_score: f64 = calc_simpheny_score::calc_simpheny_score(
                &ontology,
                terms_cleaned,
                body.gene_symbol,
                body.sim_score,
                body.num_query_genes,
                body.num_hpo_terms,
                body.data_bg,
                iterations,
                terms_url,
                genes_url,
            );

            let response = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(serde_json::to_string(&simpheny_score)
                    .unwrap_or_else(|_| "Error serializing response".to_string()));
            response
        });

    //Combine all the routes and serve them
    let routes = home
        .or(check_db) // "/check_db"
        .or(get_genes_for_term) // "/id/get_genes/{term_id}"
        .or(get_terms_for_gene) // "/gene/get_terms/{gene_id}"
        .or(get_terms_for_null_gene) // "/gene/get_terms"
        .or(get_term_by_id) // "/id/{term_id}"
        .or(get_term_by_name) // "/name/{term_name}"
        .or(get_gene_by_id) // "/gene/id/{gene_id}"
        .or(get_gene_by_name) // "/gene/name/{gene_name}"
        .or(get_genes_from_names) // "/gene/names/{gene_names}" (comma separated)
        .or(get_all_terms_ids) // "/all/terms/ids"
        .or(get_all_terms_names) // "/all/terms/names"
        .or(udn_compare) // "/compare/udn/{term_ids}" (comma separated)
        .or(orpha_compare) // "/compare/orpha/{term_ids}" (comma separated)
        .or(decipher_compare) // "/compare/decipher/{term_ids}" (comma separated)
        .or(get_orpha_population) // "/orpha_population"
        .or(get_udn_population)// "/udn_population"
        .or(get_decipher_population) // "/decipher_population"
        .or(simpheny_score); // "/simpheny_score"
    
    let cors = cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST", "OPTIONS"])
        .allow_headers(vec!["content-type"]);

    warp::serve(routes.with(cors))
//Non Production Server change to local host (docker requires the 0.0.0.0)
    .run(([127, 0, 0, 1], 8911))
        // .run(([0, 0, 0, 0], 8911))
        .await;
}

//-------------
// Database functions
//-------------

fn get_db_path() -> String {
    let db_path = String::from("/hpoAssociations/hpo.db"); //production
    // let db_path = String::from("/Users/emerson/Documents/Code/pheno_matcher_be_rust/src/hpoAssociations/hpo.db"); //development
    db_path
}

fn get_gene_by_id(db_path: String, gene_id: String) -> Result<HashMap<String, String>, rusqlite::Error> {
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare("SELECT * FROM Genes WHERE gene_id=?")?;
    let mut gene_iter = stmt.query_map([&gene_id], |row| {
        let mut gene = HashMap::new();
        gene.insert("gene_id".to_string(), row.get(0)?);
        gene.insert("gene_symbol".to_string(), row.get(1)?);
        Ok(gene)
    })?;

    //there should only be one gene returned
    let gene = match gene_iter.next() {
        Some(gene) => gene?,
        None => {
            let mut gene = HashMap::new();
            gene.insert("gene_id".to_string(), gene_id.to_string());
            gene.insert("gene_symbol".to_string(), "".to_string());
            gene
        }
    };
    Ok(gene)
}

fn get_gene_by_name(db_path: String, gene_name: String) -> Result<HashMap<String, String>, rusqlite::Error> {
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare("SELECT * FROM Genes WHERE gene_symbol COLLATE NOCASE LIKE ?")?;
    let mut gene_iter = stmt.query_map([&gene_name], |row| {
        let mut gene = HashMap::new();
        gene.insert("gene_id".to_string(), row.get(0)?);
        gene.insert("gene_symbol".to_string(), row.get(1)?);
        Ok(gene)
    })?;

    //there should only be one gene returned
    let gene = match gene_iter.next() {
        Some(gene) => gene?,
        None => {
            let mut gene = HashMap::new();
            gene.insert("gene_id".to_string(), "".to_string());
            gene.insert("gene_symbol".to_string(), gene_name.to_string());
            gene
        }
    };
    Ok(gene)
}

fn get_genes_from_names(db_path: String, gene_names: Vec<String>) -> Result<Vec<HashMap<String, String>>>{
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare("SELECT * FROM Genes WHERE gene_symbol COLLATE NOCASE LIKE ?")?;
    let mut genes: Vec<HashMap<String, String>> = Vec::new();
    for gene_name in gene_names {
        let mut gene_iter = stmt.query_map([&gene_name], |row| {
            let mut gene = HashMap::new();
            gene.insert("gene_id".to_string(), row.get(0)?);
            gene.insert("gene_symbol".to_string(), row.get(1)?);
            Ok(gene)
        })?;

        //there should only be one gene returned for each gene name
        let gene = match gene_iter.next() {
            Some(gene) => gene?,
            None => {
                let mut gene = HashMap::new();
                gene.insert("gene_id".to_string(), "".to_string());
                gene.insert("gene_symbol".to_string(), gene_name.to_string());
                gene
            }
        };
        genes.push(gene);
    }
    Ok(genes)
}

fn get_genes_for_term(db_path: String, term_id: String) -> Result<Vec<HashMap<String, String>>> {
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare(r#"
        SELECT term_to_gene.*, genes.gene_symbol, diseases.disease_name
        FROM term_to_gene 
        LEFT JOIN genes ON term_to_gene.gene_id = genes.gene_id
        JOIN diseases ON term_to_gene.disease_id = diseases.disease_id 
        WHERE term_to_gene.term_id=?"#
    )?;
    
    let gene_iter = stmt.query_map([term_id], |row| {
        let mut gene = HashMap::new();
        gene.insert("term_id".to_string(), row.get(0)?);
        gene.insert("gene_id".to_string(), row.get(1)?);
        gene.insert("frequency".to_string(), row.get(2)?);
        gene.insert("disease_id".to_string(), row.get(3)?);
        gene.insert("gene_symbol".to_string(), row.get(4)?);
        gene.insert("disease_name".to_string(), row.get(5)?);
        Ok(gene)
    })?;

    let mut genes: Vec<HashMap<String, String>> = Vec::new();
    for gene in gene_iter {
        let gene = gene?;
        genes.push(gene);
    }
    Ok(genes)
}

fn get_terms_for_gene(db_path: String, gene_id: String) -> Result<Vec<HashMap<String, String>>> {
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare(r#"
        SELECT term_to_gene.*, genes.gene_symbol, terms.name, diseases.disease_name
        FROM term_to_gene
        JOIN genes ON term_to_gene.gene_id = genes.gene_id 
        LEFT JOIN terms ON term_to_gene.term_id = terms.term_id 
        JOIN diseases ON term_to_gene.disease_id = diseases.disease_id
        WHERE term_to_gene.gene_id=?"#
    )?;
    
    let phen_iter = stmt.query_map([gene_id], |row| {
        let mut phen = HashMap::new();
        phen.insert("term_id".to_string(), row.get(0)?);
        phen.insert("gene_id".to_string(), row.get(1)?);
        phen.insert("frequency".to_string(), row.get(2)?);
        phen.insert("disease_id".to_string(), row.get(3)?);
        phen.insert("gene_symbol".to_string(), row.get(4)?);
        phen.insert("name".to_string(), row.get(5)?);
        phen.insert("disease_name".to_string(), row.get(6)?);
        Ok(phen)
    })?;

    let mut phens: Vec<HashMap<String, String>> = Vec::new();
    for phen in phen_iter {
        let phen = phen?;
        phens.push(phen);
    }
    Ok(phens)
}

fn get_term_id(db_path: String, term_id: String) -> Result<HashMap<String, String>, rusqlite::Error> {
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare("SELECT * FROM Terms WHERE term_id=?")?;
    let mut term_iter = stmt.query_map([term_id], |row| {
        let mut term = HashMap::new();
        term.insert("hpo_id".to_string(), row.get(0)?);
        term.insert("name".to_string(), row.get(1)?);
        term.insert("definition".to_string(), row.get(2)?);
        term.insert("comment".to_string(), row.get(3)?);
        term.insert("synonyms".to_string(), row.get(4)?);

        Ok(term)
    })?;

    //there should only be one term returned
    let term = term_iter.next().unwrap()?;
    Ok(term)
}

fn get_term_name(db_path: String, term_name: String) -> Result<HashMap<String, String>, rusqlite::Error> {
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare("SELECT * FROM Terms WHERE name COLLATE NOCASE LIKE ?")?;
    let mut term_iter = stmt.query_map([term_name], |row| {
        let mut term = HashMap::new();
        term.insert("hpo_id".to_string(), row.get(0)?);
        term.insert("name".to_string(), row.get(1)?);
        term.insert("definition".to_string(), row.get(2)?);
        term.insert("comment".to_string(), row.get(3)?);
        term.insert("synonyms".to_string(), row.get(4)?);

        Ok(term)
    })?;

    //there should only be one term returned
    let term = match term_iter.next() {
        Some(term) => term?,
        None => {
            let mut term = HashMap::new();
            term.insert("hpo_id".to_string(), "".to_string());
            term.insert("name".to_string(), "".to_string());
            term.insert("definition".to_string(), "".to_string());
            term.insert("comment".to_string(), "".to_string());
            term.insert("synonyms".to_string(), "".to_string());
            term
        }
    };
    Ok(term)
}

fn get_all_terms_ids(db_path: String) -> Result<HashMap<String, HashMap<String, String>>, rusqlite::Error> {
    let conn = Connection::open(db_path)?;

    let mut by_hpo_id: HashMap<String, HashMap<String, String>> = HashMap::new();

    let mut stmt = conn.prepare("SELECT * FROM Terms")?;

    let term_iter = stmt.query_map([], |row| {
        //make a new hashmap from the row itself
        let mut term: HashMap<String, String> = HashMap::new();
        term.insert("hpo_id".to_string(), row.get(0)?);
        term.insert("name".to_string(), row.get(1)?);
        term.insert("definition".to_string(), row.get(2)?);
        term.insert("comment".to_string(), row.get(3)?);
        term.insert("synonyms".to_string(), row.get(4)?);
        Ok(term)
    })?;

    for term in term_iter {
        let term = term?;
        //put the term into the hashmap with the hpo_id as the key and the term as the value
        by_hpo_id.insert(term.get("hpo_id").unwrap().to_string(), term.clone());
    }
    Ok(by_hpo_id)
}

fn get_all_terms_names(db_path: String) -> Result<HashMap<String, HashMap<String, String>>, rusqlite::Error> {
    let conn = Connection::open(db_path)?;
    let mut by_name: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut stmt = conn.prepare("SELECT * FROM Terms")?;

    let term_iter = stmt.query_map([], |row| {
        //make a new hashmap from the row itself
        let mut term: HashMap<String, String> = HashMap::new();
        term.insert("hpo_id".to_string(), row.get(0)?);
        term.insert("name".to_string(), row.get(1)?);
        term.insert("definition".to_string(), row.get(2)?);
        term.insert("comment".to_string(), row.get(3)?);
        term.insert("synonyms".to_string(), row.get(4)?);
        Ok(term)
    })?;

    for term in term_iter {
        let term = term?;
        //put the term into the hashmap with the name as the key and the term as the value
        by_name.insert(term.get("name").unwrap().to_string(), term.clone());
    }
    Ok(by_name)
}