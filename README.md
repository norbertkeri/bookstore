# Platform assignment

This repository contains:

* A bookstore web application written in rust, with accompanying tests
* Kubernetes manifest files for deploying the application, and its supporting infrastructure
* Kubernetes manifests for deploying a CI/CD system called concourse
* Pipelines for building, testing, and pushing built artifacts using said CI/CD system

Instructions:

* We assume you have a kubernetes cluster at hand, and you have an active kubectl context
* Install argocd into the cluster

    kubectl create namespace argocd
    kubectl apply -n argocd -f https://raw.githubusercontent.com/argoproj/argo-cd/stable/manifests/install.yaml

* Wait for the argocd pods to settle
* Create an argocd application that will deploy the manifests from this git repository

    argocd app create manifests --path manifests --directory-recurse --dest-server https://kubernetes.default.svc --repo https://github.com/norbertkeri/bookstore.git --sync-policy auto

* Wait for pods to settle


## ArgoCD UI
To observe things, you can access the argocd UI by grabbing the default password, and port-forwarding to the argocd pod:

    argocd admin initial-password -n argocd # username is admin
    kubectl port-forward svc/argocd-server -n argocd 8080:443

## Concourse CI/CD

You can deploy the pipelines with:

    fly -t cutters set-pipeline -c manifests/pipelines/bookstore.yml  -p bookstore

Concourse is a bit more convoluted than what I could explain in this readme, but if that is something you are interested in, you can read more on their homepage: https://concourse-ci.org/
I will be more than happy to provide a demonstration on it as well.
