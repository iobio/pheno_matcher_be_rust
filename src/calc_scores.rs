#![allow(unused_imports)]
use std::collections::HashMap;
use std::u32;
use std::sync::Arc;
use hpo::HpoTermId;
use hpo::similarity::{Similarity, StandardCombiner, GroupSimilarity};
use serde::{Deserialize, Serialize, de::IntoDeserializer};
use::hpo::{Ontology, HpoSet};
use::hpo::term::HpoGroup;
use crate::{custom_jaccard_ic, population};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ScoreReturn {
    ScoreMap(HashMap<String, HashMap<String, f32>>),
    ScoreVec(Vec<Vec<String>>),
}

pub fn calc_scores(ontology: &Arc<Ontology>, hpo_ids1: Vec<u32>, population: &Arc<HashMap<String, HashMap<String, String>>>) -> HashMap<String, ScoreReturn> {
    //Create a hashmap to store the scores
    let mut score_map: HashMap<String, HashMap<String, f32>> = HashMap::new();
    let mut score_vec: Vec<Vec<String>> = Vec::new();

    //Create a group from the hpo_ids1 vector
    let hpo_group1 = HpoGroup::from(hpo_ids1);
    let hpo_set1 = HpoSet::new(&ontology, hpo_group1);
    let sim = GroupSimilarity::new(StandardCombiner::default(), custom_jaccard_ic::CustomJaccardIC{});

    //Iterate through the population
    for (key, value) in population.iter() {
        //Create a group from the hpo_ids2 vector take the HP: off of the HPO ids and make into a u32
        let hpo_ids2: Vec<u32> = value.get("Terms").unwrap().split("; ").map(|s| s.replace("HP:", "").parse::<u32>().unwrap()).collect();
        let hpo_group2 = HpoGroup::from(hpo_ids2);
        let hpo_set2 = HpoSet::new(&ontology, hpo_group2);
        //Calculate the similarity
        let similarity = sim.calculate(&hpo_set1, &hpo_set2);
        let mut current_sim = HashMap::new();
        current_sim.insert("score".to_string(), similarity);
        //Add the similarity to the score_map
        score_map.insert(key.to_string(), current_sim);
        //Add the similarity to the score_vec
        score_vec.push(vec![key.to_string(), similarity.to_string()]);
    }
    //use the score_map to create a ranked_map
    let return_map = create_ranked_vec(score_map, score_vec);
    //Return the ranked_map
    return_map
}

fn create_ranked_vec(score_map: HashMap<String, HashMap<String, f32>>, score_vec: Vec<Vec<String>>) -> HashMap<String, ScoreReturn> {
    //The score map will be modified to add rank as we determine the rank from the ordering of the scores
    let mut score_map = score_map;
    //Create a vector to store the ranked individuals
    let mut ranked_vec: Vec<Vec<String>> = score_vec.clone();
    //Sort the score_vec by the score
    ranked_vec.sort_by(|a, b| b[1].partial_cmp(&a[1]).unwrap());

    //Iterate through the ranked_vec get the rank and add it to the score_map based on the key and [0]
    for (i, v) in ranked_vec.iter().enumerate() {
        let mut current_rank = HashMap::new();
        current_rank.insert("rank".to_string(), (i + 1) as f32);
        score_map.get_mut(&v[0]).unwrap().insert("rank".to_string(), (i + 1) as f32);
    }
    //Create a hashmap to store the score_map and ranked_vec
    let mut return_map: HashMap<String, ScoreReturn> = HashMap::new();
    return_map.insert("score_map".to_string(), ScoreReturn::ScoreMap(score_map));
    return_map.insert("ranked_vec".to_string(), ScoreReturn::ScoreVec(ranked_vec));
    return_map
}