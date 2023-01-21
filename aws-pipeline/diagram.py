from diagrams import Cluster, Diagram, Edge
from diagrams.aws.compute import Lambda
from diagrams.aws.integration import SQS
from diagrams.aws.storage import S3

# from diagrams.onprem.compute import Server
# from diagrams.onprem.database import PostgreSQL
# from diagrams.onprem.inmemory import Redis

with Diagram(name="PSAPI Indexer on AWS", show=True, graph_attr={"layout": "dot"}):

    queue = SQS("Change ID Task Queue")
    worker = Lambda("Task Worker")
    bucket = S3("S3 IA Bucket")

    Edge(label="Single manual trigger at the start") >> queue
    worker >> Edge(label="on new task") >> queue
    queue >> Edge(label="register new task") >> worker
    worker >> Edge(label="store chunk of snapshots") >> bucket
