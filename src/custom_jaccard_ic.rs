use hpo::HpoTerm;
use hpo::similarity::Similarity;

pub struct CustomJaccardIC {}

impl Similarity for CustomJaccardIC {
    fn calculate(&self, a: &HpoTerm, b: &HpoTerm) -> f32 {
        //get the sum of the information content of each term in the union and intersection
        let union = a.union_ancestors(b);
        let union_iter = union.iter();
        let intersection = a.common_ancestors(b);
        let intersection_iter = intersection.iter();

        let mut union_sum = 0.0;
        let mut intersection_sum = 0.0;

        let mut found_a = false;
        let mut found_b = false;

        for term in union_iter {
            union_sum += term.information_content().omim_disease();
            //if we find a or b in the union set found_a or found_b to true
            if term == *a {
                found_a = true;
            }
            if term == *b {
                found_b = true;
            }
        }

        for term in intersection_iter {
            intersection_sum += term.information_content().omim_disease();
        }

        //If the terms are the same the similarity is 1.0
        if a == b {
            return 1.0;
        }
        //if the union is 0, the similarity is 0.0
        if union_sum == 0.0 {
            return 0.0;
        }
        
        if found_a {
            union_sum += b.information_content().omim_disease();
        } else if found_b {
            union_sum += a.information_content().omim_disease();
        } else {
            union_sum += a.information_content().omim_disease() + b.information_content().omim_disease();
        };

        //Return the similarity
        return intersection_sum / union_sum;
    }
}