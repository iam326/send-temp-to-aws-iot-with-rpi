import * as cdk from '@aws-cdk/core';
import * as dynamodb from '@aws-cdk/aws-dynamodb';
import * as iot from '@aws-cdk/aws-iot';
import * as iam from '@aws-cdk/aws-iam';

import { BillingMode } from '@aws-cdk/aws-dynamodb/lib/table';

const projectName = 'send-temp-to-aws-iot-with-rpi';
const ioTCertificateArn = process.env.AWS_IOT_CERTIFICATE_ARN as string;
if (!ioTCertificateArn) {
  throw Error('iot certificate arn is undefined.');
}

export class BackendStack extends cdk.Stack {
  constructor(scope: cdk.Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    const table = new dynamodb.Table(this, `${projectName}-table`, {
      partitionKey: {
        name: 'timestamp',
        type: dynamodb.AttributeType.NUMBER
      },
      billingMode: BillingMode.PAY_PER_REQUEST,
      tableName: `${projectName}-table`
    });

    const tablePutItemRole = new iam.CfnRole(this, `${projectName}-table-put-item-role`, {
      assumeRolePolicyDocument: {
        Statement: [
          {
            Action: 'sts:AssumeRole',
            Effect: 'Allow',
            Principal: {
              Service: 'iot.amazonaws.com'
            }
          }
        ],
        Version: '2012-10-17'
      },
      policies: [
        {
          policyName: `${projectName}-table-put-item-policy`,
          policyDocument: {
            Version: '2012-10-17',
            Statement: [
              {
                Effect: 'Allow',
                Action: 'dynamoDB:PutItem',
                Resource: table.tableArn
              }
            ]
          }
        }
      ]
    });

    const iotThing = new iot.CfnThing(this, `${projectName}-iot-thing`, {
      thingName: `${projectName}-thing`
    });

    const iotPolicy = new iot.CfnPolicy(this, `${projectName}-iot-policy`, {
      policyDocument: {
        Version: '2012-10-17',
        Statement: [
          {
            Effect: 'Allow',
            Action: 'iot:*',
            Resource: '*'
          }
        ]
      },
      policyName: `${projectName}-iot-policy`
    });

    const iotThingPrincipalAttachment = new iot.CfnThingPrincipalAttachment(
      this,
      `${projectName}-iot-thing-principal-attachment`,
      {
        principal: ioTCertificateArn,
        thingName: `${projectName}-thing`
      }
    );
    iotThingPrincipalAttachment.addDependsOn(iotThing);

    const iotPolicyPrincipalAttachment = new iot.CfnPolicyPrincipalAttachment(
      this,
      `${projectName}-iot-policy-principal-attachment`,
      {
        principal: ioTCertificateArn,
        policyName: `${projectName}-iot-policy`
      }
    );
    iotPolicyPrincipalAttachment.addDependsOn(iotPolicy);

    const IoTTopicRule = new iot.CfnTopicRule(this, `${projectName}-iot-topic-rule`, {
      ruleName: `${projectName.split('-').join('_')}_iot_topic_rule`,
      topicRulePayload: {
        actions: [
          {
            dynamoDBv2: {
              putItem: {
                tableName: table.tableName
              },
              roleArn: tablePutItemRole.attrArn
            }
          }
        ],
        awsIotSqlVersion: '2016-03-23',
        ruleDisabled: false,
        sql: 'SELECT * FROM \'iot/topic\'',
      }
    });
  }
}

const app = new cdk.App();
new BackendStack(app, `${projectName}-stack`);
app.synth();
