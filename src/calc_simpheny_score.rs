#![allow(unused_imports)]
use crate::{custom_jaccard_ic, population};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use rand::prelude::*;
use rand::rng;
use statrs::distribution::{ChiSquared, ContinuousCDF};
use hpo::similarity::{Similarity, StandardCombiner, GroupSimilarity};
use hpo::{Ontology, HpoSet, HpoTermId};
use hpo::term::HpoGroup;
use csv::Reader;

// Using "top down" approach to organization; public function at the top, private functions below
#[allow(unused_variables, unused_imports)]
pub fn calc_simpheny_score(ontology: &Arc<Ontology>, hit_terms: Vec<u32>, hit_gene: String, sim_score: f32, num_query_genes: u32, num_hpo_terms: u32, data_bg: String, iterations: u32, terms_url: &str, genes_url: &str) -> f64 {
    let mut num_terms = num_hpo_terms;
    
    // If the data_bg is udn use udn if it is clinvar then use clinvar otherwise default to udn
    let (scale, dof);
    if data_bg == "udn" {
            // UDN values
        scale = 1.0703270447328037;
        dof = 3.737175491999793;
    } else if data_bg == "clinvar" {
        // ClinVar values
        scale = 1.0334745692972533;
        dof = 3.8704387305049393;
    } 

    // Grab all the gene_symbols from the database
    let mut gene_reader = Reader::from_path(genes_url).unwrap();
    let all_gene_list: Vec<String> = gene_reader
        .records()
        .skip(1) // Skip header row
        .filter_map(|result| result.ok())
        .map(|record| record[0].to_string()) // Assuming the first column contains gene symbols
        .collect();

    // Grab all the hpo_terms from the database
    let mut term_reader = Reader::from_path(terms_url).unwrap();
    let all_term_list: Vec<u32> = term_reader
        .records()
        .skip(1) // Skip header row
        .filter_map(|result| result.ok())
        .filter_map(|record| {
            record[0]
                .strip_prefix("HP:")
                .and_then(|s| s.parse::<u32>().ok())
        })
        .filter(|term_id| {
            let tid = HpoTermId::from(*term_id);
            ontology.hpo(tid).is_some()
        }) // Ensure the term exists in the ontology
        .collect();

    if num_terms > 10 {
        // Truncate for computational efficiency
        num_terms = 10;
    }

    let (pheno_p, gene_p) = calc_p_vals(ontology, hit_terms, hit_gene, sim_score, all_term_list, all_gene_list, num_terms, num_query_genes, iterations);
    let combined_p = empirical_browns_method(pheno_p, gene_p, scale, dof);

    -combined_p.log10() // Return the SimPheny score
}

fn calc_p_vals(ontology: &Arc<Ontology>, hit_terms: Vec<u32>, hit_gene: String, sim_score: f32, all_term_list: Vec<u32>, all_gene_list: Vec<String>, num_hpo_terms: u32, num_query_genes: u32, iterations: u32) -> (f64, f64) {
    let mut i: u32 = 0;
    let mut rand_genes: Vec<String> = Vec::new();
    let mut rand_scores: Vec<f32> = Vec::new();

    //Convert the ids into a group and then set them
    let hit_group = HpoGroup::from(hit_terms);
    let hit_set = HpoSet::new(&ontology, hit_group);

    while i < iterations {
        // Randomly sample terms
        let mut random_terms: Vec<u32> = all_term_list
            .choose_multiple(&mut rng(), num_hpo_terms as usize)
            .cloned()
            .collect::<HashSet<_>>() // Convert to HashSet to ensure uniqueness
            .into_iter() // Convert to an iterator
            .collect::<Vec<u32>>(); // Collect back into a Vec<u32>

        // Ensure we have unique terms
        while random_terms.len() < num_hpo_terms as usize {
            let sample_num = num_hpo_terms as usize - random_terms.len();

            // Sampling more terms (consumed by .extend below)
            let more_terms: Vec<u32> = all_term_list
                .choose_multiple(&mut rng(), sample_num)
                .cloned()
                .collect::<HashSet<_>>()
                .into_iter()
                .collect::<Vec<u32>>();

            random_terms.extend(more_terms); // Extend the random_terms with more unique terms we will do again if we still don't have enough
        }

        let random_group = HpoGroup::from(random_terms);
        let random_set = HpoSet::new(&ontology, random_group);

        // Calculate the similarity score
        let sim = GroupSimilarity::new(StandardCombiner::default(), custom_jaccard_ic::CustomJaccardIC{});
        let score = sim.calculate(&hit_set, &random_set);
        rand_scores.push(score);

        // Randomly sample genes (will be consumed by .extend below)
        let random_gene_sample: Vec<String> = all_gene_list
            .choose_multiple(&mut rng(), num_query_genes as usize)
            .cloned()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<String>>();

        // Gene sample is already unique, so we can just extend
        rand_genes.extend(random_gene_sample);

        i += 1;
    }

    // Calculate p-values
    let pheno_pval = (rand_scores.iter().filter(|&s| s >= &sim_score).count() as f64 + 1.0) / (rand_scores.len() as f64 + 1.0);
    let gene_pval = (rand_genes.iter().filter(|&g| g == &hit_gene).count() as f64 + 1.0) / (iterations as f64 + 1.0);

    (pheno_pval, gene_pval)
}


fn empirical_browns_method(pheno_pval: f64, gene_pval: f64, scale: f64, dof: f64) -> f64 {
    let stat = -2.0 * (pheno_pval.ln() + gene_pval.ln()); // ln is the natural logarithm which should be the same as np.log in Python
    let adjusted_stat = stat / scale;

    let chi2_dist = ChiSquared::new(dof).unwrap(); // Should maybe handle if there is an error in creating the distribution instead of unwrap
    let combined_p = 1.0 - chi2_dist.cdf(adjusted_stat);

    combined_p
}